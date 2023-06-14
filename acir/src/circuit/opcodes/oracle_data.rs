use crate::native_types::{Expression, Witness};
use acir_field::FieldElement;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleData {
    /// Name of the oracle
    pub name: String,
    /// Inputs
    pub inputs: Vec<Expression>,
    /// Input values - they are progressively computed by the pwg
    pub input_values: Vec<FieldElement>,
    /// Output witness
    pub outputs: Vec<Witness>,
    /// Output values - they are computed by the (external) oracle once the input_values are known
    pub output_values: Vec<FieldElement>,
}

impl std::fmt::Display for OracleData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ORACLE: {}", self.name)?;
        let solved = if self.input_values.len() == self.inputs.len() { "solved" } else { "" };

        if !self.inputs.is_empty() {
            write!(
                f,
                "Inputs: _{}..._{}{solved}",
                self.inputs.first().unwrap(),
                self.inputs.last().unwrap()
            )?;
        }

        let solved = if self.output_values.len() == self.outputs.len() { "solved" } else { "" };
        write!(
            f,
            "Outputs: _{}..._{}{solved}",
            self.outputs.first().unwrap().witness_index(),
            self.outputs.last().unwrap().witness_index()
        )
    }
}
