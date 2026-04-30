use crate::nfs::types::{
    CallbackSecurityParameters, ChannelAttrs, ClientId, CreateSessionArgs,
    SequenceId,
};

#[derive(Debug)]
pub struct Session {
    pub client_id: ClientId,
    pub sequence: SequenceId,
    pub flags: u32,
    pub fore_channel_attrs: ChannelAttrs,
    pub back_channel_attrs: ChannelAttrs,
    pub callback_program: u32,
    pub security_parameters: Vec<CallbackSecurityParameters>,
    pub slots: Vec<Option<SequenceId>>,
}

impl Session {
    pub fn new(args: CreateSessionArgs) -> Self {
        let num_slots = args.fore_channel_attrs.max_requests;
        Self {
            client_id: args.client_id,
            sequence: args.sequence,
            flags: 0,
            fore_channel_attrs: args.fore_channel_attrs,
            back_channel_attrs: args.back_channel_attrs,
            callback_program: args.callback_program,
            security_parameters: args.security_parameters,
            slots: vec![None; num_slots as usize],
        }
    }
}
