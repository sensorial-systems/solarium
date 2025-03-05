use crate::prelude::*;
use crate::Program;
use crate::Workspace;

#[derive(Debug, Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct Idl(pub anchor_lang_idl_spec::Idl);

impl Idl {
    pub fn deploy(&self, _workspace: &Workspace) -> Result<()> {
        Ok(())
    }
}

impl TryFrom<&Program> for Idl {
    type Error = anyhow::Error;

    fn try_from(program: &Program) -> Result<Self, Self::Error> {
        let anchor_idl = anchor_lang_idl_spec::Idl {
            address: program.public_key.to_string(),
            metadata: anchor_lang_idl_spec::IdlMetadata {
                contact: None,
                description: None,
                name: program.name.to_string(),
                repository: None,
                spec: anchor_lang_idl_spec::IDL_SPEC.to_string(),
                version: "0.1.0".to_string(),
                dependencies: vec![],
                deployments: None,
            },
            docs: vec![],
            instructions: vec![],
            accounts: vec![],
            events: vec![],
            errors: vec![],
            types: vec![],
            constants: vec![],
        };
        Ok(Idl(anchor_idl))
    }
}