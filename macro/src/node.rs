use proc_macro2::TokenStream;
use syn::*;
use quote::quote;
use convert_case::{Case, Casing};

pub type NodeId = u32;

pub fn expand(
    state: &ItemStruct,
) -> Result<TokenStream> {    
    let mp = quote!{ frand_node::macro_prelude };

    let vis = &state.vis;
    let state_name = state.ident.clone();

    let message_name = Ident::new(
        &format!("{}Message", state_name.to_string()).to_case(Case::Pascal), 
        state_name.span(),
    );

    let consensus_name = Ident::new(
        &format!("{}Consensus", state_name.to_string()).to_case(Case::Pascal), 
        state_name.span(),
    );

    let node_name = Ident::new(
        &format!("{}Node", state_name.to_string()).to_case(Case::Pascal), 
        state_name.span(),
    );

    let fields: Vec<&Field> = match &state.fields {
        Fields::Named(fields_named) => fields_named.named.iter().collect(),
        _ => unimplemented!("not supported"),
    };  

    let viss: Vec<_> = fields.iter().map(|field| &field.vis).collect();
    let indexes: Vec<_> = (0..fields.len() as NodeId).into_iter().collect();
    let names: Vec<_> = fields.iter().filter_map(|field| field.ident.as_ref()).collect();
    let tys: Vec<_> = fields.iter().map(|field| &field.ty).collect();

    let message_tys: Vec<_> = tys.iter().map(|ty| 
        quote!{ <#ty as #mp::State>::Message }
    ).collect();

    let consensus_tys: Vec<_> = tys.iter().map(|ty| 
        quote!{ <#ty as #mp::State>::Consensus }
    ).collect();

    let node_tys: Vec<_> = tys.iter().map(|ty| 
        quote!{ <#ty as #mp::State>::Node }
    ).collect();

    Ok(quote!{
        #[derive(Debug, Clone)]
        #vis enum #message_name {
            #(#[allow(non_camel_case_types)] #names(#[allow(dead_code)] #message_tys),)*
            #[allow(non_camel_case_types)] State(#[allow(dead_code)] #state_name),
        }

        #[derive(Debug, Clone)]
        #vis struct #consensus_name<M: #mp::Message> {
            key: #mp::NodeKey,
            #(#viss #names: #consensus_tys<M>,)*
        }

        #[derive(Debug, Clone)]
        #vis struct #node_name<M: #mp::Message> {
            key: #mp::NodeKey,
            callback: #mp::Callback<M>,
            future_callback: #mp::FutureCallback<M>,
            #(#viss #names: #node_tys<M>,)*
        }

        impl #mp::State for #state_name {
            type Message = #message_name;
            type Consensus<M: Message> = #consensus_name<M>;
            type Node<M: Message> = #node_name<M>;

            fn apply(
                &mut self,  
                message: Self::Message,
            ) {
                match message {
                    #(Self::Message::#names(message) => self.#names.apply(message),)*
                    Self::Message::State(state) => *self = state,
                }
            }
        }

        impl #mp::Message for #message_name {
            fn from_state<S: #mp::State>(
                header: &#mp::Header, 
                depth: usize, 
                state: S,
            ) -> core::result::Result<Self, #mp::MessageError> {
                Ok(match header.get(depth).copied() {
                    #(Some(#indexes) => Ok(
                        #message_name::#names(#message_tys::from_state(header, depth + 1, state)?)
                    ),)*
                    Some(_) => Err(#mp::MessageError::new(
                        header.clone(),
                        depth,
                        "unknown id",
                    )),
                    None => Ok(Self::State(
                        unsafe { Self::cast_state(state) }
                    )),
                }?)     
            }

            fn from_packet(
                packet: &#mp::Packet, 
                depth: usize, 
            ) -> core::result::Result<Self, #mp::PacketError> {
                Ok(match packet.get_id(depth) {
                    #(Some(#indexes) => Ok(
                        #message_name::#names(#message_tys::from_packet(packet, depth + 1)?)
                    ),)*
                    Some(_) => Err(#mp::PacketError::new(
                        packet.clone(),
                        depth,
                        "unknown id",
                    )),
                    None => Ok(Self::State(
                        packet.read_state()
                    )),
                }?)     
            }

            fn to_packet(
                &self,
                header: &#mp::Header, 
            ) -> core::result::Result<#mp::Packet, #mp::MessageError> {       
                match self {
                    #(Self::#names(message) => message.to_packet(header),)*
                    Self::State(state) => Ok(#mp::Packet::new(header.clone(), state)),
                }
            }
        }

        impl<M: #mp::Message> Default for #consensus_name<M> {      
            fn default() -> Self { Self::new(vec![], None) }
        }

        impl<M: #mp::Message> #mp::Consensus<M, #state_name> for #consensus_name<M> {  
            fn new(
                mut key: Vec<#mp::NodeId>,
                id: Option<#mp::NodeId>,
            ) -> Self {
                if let Some(id) = id { key.push(id); }
                
                Self { 
                    key: key.clone().into_boxed_slice(),   
                    #(#names: #mp::Consensus::new(key.clone(), Some(#indexes)),)*                     
                }
            }
    
            fn new_node(
                &self, 
                callback: &#mp::Callback<M>, 
                future_callback: &#mp::FutureCallback<M>,
            ) -> #node_name<M> {
                #node_name::new_from(self, callback, future_callback)
            }
            
            fn clone_state(&self) -> #state_name { 
                #state_name {
                    #(#names: self.#names.clone_state(),)*   
                }
            }

            fn apply(&mut self, message: #message_name) {
                match message {
                    #(#message_name::#names(#names) => self.#names.apply(#names),)*
                    #message_name::State(state) => self.apply_state(state),
                } 
            }

            fn apply_state(&mut self, state: #state_name) {
                #(self.#names.apply_state(state.#names);)*       
            }
        }

        impl<M: #mp::Message> #mp::Node<M, #state_name> for #node_name<M> { 
            type State = #state_name;

            fn key(&self) -> &#mp::NodeKey { &self.key }

            fn new_from(
                consensus: &#consensus_name<M>,
                callback: &#mp::Callback<M>, 
                future_callback: &#mp::FutureCallback<M>,
            ) -> Self {
                Self { 
                    key: consensus.key.clone(),
                    callback: callback.clone(), 
                    future_callback: future_callback.clone(), 
                    #(#names: #mp::Node::new_from(&consensus.#names, callback, future_callback),)*  
                }
            }

            fn clone_state(&self) -> #state_name { 
                #state_name {
                    #(#names: self.#names.clone_state(),)*   
                }
            }
        }

        impl<M: #mp::Message> #mp::Emitter<M, #state_name> for #node_name<M> {  
            fn emit(&self, state: #state_name) {
                self.callback.emit(self.key.clone(), state)
            }

            fn emit_future<Fu>(&self, future: Fu) 
            where Fu: 'static + std::future::Future<Output = #state_name> + Send {
                self.future_callback.emit(self.key.clone(), future)
            }
        }
    })
}