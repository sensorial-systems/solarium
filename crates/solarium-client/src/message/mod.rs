use solana_sdk::instruction::Instruction;

#[derive(Default)]
pub struct Message {
    pub instructions: Vec<Instruction>,
}

impl Message {
    pub fn new(instructions: impl Into<Vec<Instruction>>) -> Self {
        let instructions = instructions.into();
        Self { instructions }
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }
}
