use crate::egress_path::EgressPathConfig;
use crate::egress_source::{EgressPool, EgressSource};
use crate::http_server::HttpListenerParams;
use crate::lifecycle::LifeCycle;
use crate::logging::{ClassifierParams, LogFileParams};
use crate::queue::QueueConfig;
use crate::runtime::spawn;
use crate::smtp_server::{EsmtpListenerParams, RejectError};
use config::{any_err, get_or_create_module};
use mlua::{Function, Lua, LuaSerdeExt, Value};
use mod_redis::RedisConnKey;
use serde::Deserialize;
use std::path::PathBuf;

pub fn register(lua: &Lua) -> anyhow::Result<()> {
    let kumo_mod = get_or_create_module(lua, "kumo")?;

    kumo_mod.set(
        "on",
        lua.create_function(move |lua, (name, func): (String, Function)| {
            let decorated_name = format!("kumomta-on-{}", name);
            lua.set_named_registry_value(&decorated_name, func)?;
            Ok(())
        })?,
    )?;

    kumo_mod.set(
        "configure_bounce_classifier",
        lua.create_function(move |lua, params: Value| {
            let params: ClassifierParams = lua.from_value(params)?;
            params.register().map_err(any_err)
        })?,
    )?;

    kumo_mod.set(
        "configure_local_logs",
        lua.create_function(move |lua, params: Value| {
            let params: LogFileParams = lua.from_value(params)?;
            crate::logging::Logger::init(params).map_err(any_err)
        })?,
    )?;

    kumo_mod.set(
        "start_http_listener",
        lua.create_async_function(|lua, params: Value| async move {
            let params: HttpListenerParams = lua.from_value(params)?;
            params.start().await.map_err(any_err)?;
            Ok(())
        })?,
    )?;

    kumo_mod.set(
        "start_esmtp_listener",
        lua.create_async_function(|lua, params: Value| async move {
            let params: EsmtpListenerParams = lua.from_value(params)?;
            spawn("start_esmtp_listener", async move {
                if let Err(err) = params.run().await {
                    tracing::error!("Error in SmtpServer: {err:#}");
                }
            })
            .map_err(any_err)?;
            Ok(())
        })?,
    )?;

    kumo_mod.set(
        "define_spool",
        lua.create_async_function(|lua, params: Value| async move {
            let params = lua.from_value(params)?;
            spawn("define_spool", async move {
                if let Err(err) = define_spool(params).await {
                    tracing::error!("Error in spool: {err:#}");
                    LifeCycle::request_shutdown().await;
                }
            })
            .map_err(any_err)?
            .await
            .map_err(any_err)
        })?,
    )?;

    kumo_mod.set(
        "configure_redis_throttles",
        lua.create_async_function(|lua, params: Value| async move {
            let key: RedisConnKey = lua.from_value(params)?;
            let conn = key.open().await.map_err(any_err)?;
            throttle::use_redis(conn).map_err(any_err)
        })?,
    )?;

    kumo_mod.set(
        "reject",
        lua.create_function(move |_lua, (code, message): (u16, String)| {
            Err::<(), mlua::Error>(mlua::Error::external(RejectError { code, message }))
        })?,
    )?;

    kumo_mod.set(
        "make_egress_path",
        lua.create_function(move |lua, params: Value| {
            let config: EgressPathConfig = lua.from_value(params)?;
            Ok(config)
        })?,
    )?;

    kumo_mod.set(
        "make_queue_config",
        lua.create_function(move |lua, params: Value| {
            let config: QueueConfig = lua.from_value(params)?;
            Ok(config)
        })?,
    )?;

    kumo_mod.set(
        "define_egress_source",
        lua.create_function(move |lua, params: Value| {
            let source: EgressSource = lua.from_value(params)?;
            source.register();
            Ok(())
        })?,
    )?;
    kumo_mod.set(
        "define_egress_pool",
        lua.create_function(move |lua, params: Value| {
            let pool: EgressPool = lua.from_value(params)?;
            pool.register().map_err(any_err)
        })?,
    )?;

    Ok(())
}

#[derive(Deserialize)]
pub enum SpoolKind {
    LocalDisk,
    RocksDB,
}
impl Default for SpoolKind {
    fn default() -> Self {
        Self::LocalDisk
    }
}

#[derive(Deserialize)]
pub struct DefineSpoolParams {
    pub name: String,
    pub path: PathBuf,
    #[serde(default)]
    pub kind: SpoolKind,
    #[serde(default)]
    pub flush: bool,
}

async fn define_spool(params: DefineSpoolParams) -> anyhow::Result<()> {
    crate::spool::SpoolManager::get()
        .await
        .new_local_disk(params)
}
