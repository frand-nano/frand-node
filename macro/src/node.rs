use proc_macro2::TokenStream;
use syn::*;
use quote::quote;
use convert_case::{Case, Casing};

pub type AnchorId = u32;

pub fn expand(
    state: &ItemStruct,
) -> Result<TokenStream> {    
    let mp = quote!{ frand_node::macro_prelude };

    let vis = &state.vis;
    let state_name = state.ident.clone();

    let anchor_name = Ident::new(
        &format!("{}Anchor", state.ident.to_string()).to_case(Case::Pascal), 
        state.ident.span(),
    );

    let message_name = Ident::new(
        &format!("{}Message", state_name.to_string()).to_case(Case::Pascal), 
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
    let indexes: Vec<_> = (0..fields.len() as AnchorId).into_iter().collect();
    let names: Vec<_> = fields.iter().filter_map(|field| field.ident.as_ref()).collect();
    let tys: Vec<_> = fields.iter().map(|field| &field.ty).collect();

    let anchor_tys: Vec<_> = tys.iter().map(|ty| 
        quote!{ <#ty as #mp::State>::Anchor }
    ).collect();

    let message_tys: Vec<_> = tys.iter().map(|ty| 
        quote!{ <#ty as #mp::State>::Message }
    ).collect();

    let node_tys: Vec<_> = tys.iter().map(|ty| 
        quote!{ <#ty as #mp::State>::Node }
    ).collect();

    Ok(quote!{
        #[derive(Debug, Clone)]
        #vis struct #anchor_name {
            key: #mp::AnchorKey,
            reporter: #mp::Reporter,
            #(#viss #names: #anchor_tys,)*
        }

        #[derive(Debug, Clone)]
        #vis enum #message_name {
            #(#[allow(non_camel_case_types)] #names(#[allow(dead_code)] #message_tys),)*
            #[allow(non_camel_case_types)] State(#[allow(dead_code)] #state_name),
        }

        #vis struct #node_name<'n> {
            anchor: &'n #anchor_name,
            #(#viss #names: #node_tys<'n>,)*
        }

        impl #mp::State for #state_name {
            type Anchor = #anchor_name;
            type Message = #message_name;
            type Node<'n> = #node_name<'n>;

            fn apply(
                &mut self, 
                depth: usize, 
                packet: #mp::Packet,
            ) -> #mp::anyhow::Result<()> {
                match packet.get_id(depth) {
                    #(Some(#indexes) => self.#names.apply(depth+1, packet),)*
                    Some(_) => Err(packet.error(depth, "unknown id")),
                    None => Ok(*self = packet.read_state()),
                }
            }    

            fn apply_message(
                &mut self,  
                message: #message_name,
            ) {
                match message {
                    #(#message_name::#names(message) => self.#names.apply_message(message),)*
                    #message_name::State(state) => *self = state,
                }
            }
        }

        impl #mp::Anchor for #anchor_name {
            fn key(&self) -> &#mp::AnchorKey { &self.key }
            fn reporter(&self) -> &#mp::Reporter { &self.reporter }
        
            fn new(
                mut key: Vec<#mp::AnchorId>,
                id: Option<#mp::AnchorId>,
                reporter: &#mp::Reporter,
            ) -> Self {
                if let Some(id) = id { key.push(id); }
        
                Self { 
                    #(#names: #anchor_tys::new(key.clone(), Some(#indexes), reporter),)*   
                    key: key.into_boxed_slice(),      
                    reporter: reporter.clone(),
                }
            }
        }

        impl Default for #anchor_name {
            fn default() -> Self { Self::new(vec![], None, &#mp::Reporter::None) }
        }

        impl #mp::Emitter for #anchor_name {
            fn emit<E: 'static + #mp::Emitable>(&self, emitable: E) {
                self.reporter().report(self.key(), emitable)
            }
            
            fn emit_future<Fu, E>(&self, future: Fu) 
            where 
            Fu: 'static + std::future::Future<Output = E> + Send,
            E: 'static + #mp::Emitable + Sized,
            {
                self.reporter().report_future(self.key(), future)
            }
        }

        impl #mp::Message for #message_name {
            fn from_packet(depth: usize, packet: &#mp::Packet) -> #mp::anyhow::Result<Self> {
                Ok(match packet.get_id(depth) {
                    #(Some(#indexes) => Ok(
                        #message_name::#names(#message_tys::from_packet(depth + 1, packet)?)
                    ),)*
                    Some(_) => Err(packet.error(depth, "unknown id")),
                    None => Ok(Self::State(packet.read_state())),
                }?)     
            }
        }
                
        impl #mp::Emitter for #node_name<'_> {
            fn emit<E: 'static + #mp::Emitable>(&self, emitable: E) { 
                self.anchor.emit(emitable) 
            }    
        
            fn emit_future<Fu, E>(&self, future: Fu) 
            where 
            Fu: 'static + std::future::Future<Output = E> + Send,
            E: 'static + #mp::Emitable + Sized,
            {
                self.anchor.emit_future(future) 
            }
        }

        impl<'n> #mp::Node<'n, #state_name> for #node_name<'n> {
            fn new(state: &'n #state_name, anchor: &'n #anchor_name) -> Self { 
                Self { 
                    anchor, 
                    #(#names: #node_tys::new(&state.#names, &anchor.#names),)*   
                } 
            } 

            fn clone_state(&self) -> #state_name { 
                #state_name {
                    #(#names: self.#names.clone_state(),)*   
                }
            }

            fn apply(
                &mut self, 
                depth: usize, 
                packet: &#mp::Packet,
            ) -> #mp::anyhow::Result<()> {
                match packet.get_id(depth) {
                    #(Some(#indexes) => self.#names.apply(depth+1, packet),)*
                    Some(_) => Err(packet.error(depth, "unknown id")),
                    None => Ok(self.apply_state(packet.read_state())),
                } 
            }
            
            fn apply_export(
                &mut self, 
                depth: usize, 
                packet: &#mp::Packet,
            ) -> #mp::anyhow::Result<#message_name> {
                match packet.get_id(depth) {
                    #(Some(#indexes) => {
                        Ok(#message_name::#names(self.#names.apply_export(depth+1, packet)?))
                    },)*
                    Some(_) => Err(packet.error(depth, "unknown id")),
                    None => {
                        let state: #state_name = packet.read_state();    
                        self.apply_state(state.clone());       
                        Ok(#message_name::State(state))
                    },
                }        
            }
        }

        impl #node_name<'_> {
            pub fn apply_state(&mut self, state: #state_name) {
                #(self.#names.apply_state(state.#names);)*       
            }
        }
    })
}