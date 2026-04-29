use crate::nfs::types::{
    CallbackSecurityParameters, ChannelAttrs, ClientId, CreateSessionArgs,
    SequenceId,
};

#[derive(Debug)]
pub struct Session {
    client_id: ClientId,
    sequence: SequenceId,
    flags: u32,
    fore_channel_attrs: ChannelAttrs,
    back_channel_attrs: ChannelAttrs,
    callback_program: u32,
    security_parameters: Vec<CallbackSecurityParameters>,
}

impl Session {
    pub fn new(args: CreateSessionArgs) -> Self {
        Self {
            client_id: args.client_id,
            sequence: args.sequence,
            flags: 0,
            fore_channel_attrs: args.fore_channel_attrs,
            back_channel_attrs: args.back_channel_attrs,
            callback_program: args.callback_program,
            security_parameters: args.security_parameters,
        }
    }
}
