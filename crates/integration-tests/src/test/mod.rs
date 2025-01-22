#![cfg(test)]
mod auth_deliver;
mod auth_deliver_invalid_password;
mod disconnect_in_mail_from;
mod end_to_end;
mod end_to_end_deferred_queue;
mod end_to_end_stuffed;
mod end_to_end_webhook;
mod end_to_end_webhook_batch;
mod log_oob_arf;
mod max_line_length;
mod perm_fail;
mod rebind;
mod rebind_event_defined;
mod rebind_event_missing;
mod retry_schedule;
mod spf_basic;
mod suspend_delivery_ready_q;
mod suspend_delivery_ready_q_and_deliver;
mod suspend_delivery_scheduled_q;
mod suspend_delivery_scheduled_q_and_deliver;
mod temp_fail;
mod tls_opportunistic_fail;
mod tls_opportunistic_reconnect;
mod tsa_basic_automation;
mod tsa_bounce_automation;
mod tsa_bounce_campaign;
mod tsa_bounce_tenant;
mod tsa_campaign_suspension;
mod tsa_tenant_suspension;
mod tsa_tenant_suspension_issue290;
