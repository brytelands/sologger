use serde_derive::{Deserialize, Serialize};
use {log::*, std::collections::HashSet};

///ProgramsSelector maintains a list of program IDs that are to be monitored through the log subscription output.
///If all programs are to be subscribed to (not recommended), then set the value of programs to "*".
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramsSelector {
    pub programs: HashSet<Vec<u8>>,
    pub select_all_programs: bool,
}

impl ProgramsSelector {
    /// Creates a new ProgramsSelector that will select all programs
    pub fn new_all_programs() -> Self {
        Self::new(&["*".to_string()])
    }

    /// Creates a new ProgramsSelector that will select the specified programs
    pub fn new(programs: &[String]) -> Self {
        info!("Creating ProgramsSelector from programs: {:?}", programs);

        let select_all_programs = programs.iter().any(|key| key == "*");
        if select_all_programs {
            return ProgramsSelector {
                programs: HashSet::default(),
                select_all_programs,
            };
        }
        let programs = programs
            .iter()
            .map(|key| bs58::decode(key).into_vec().unwrap())
            .collect();
        ProgramsSelector {
            programs,
            select_all_programs,
        }
    }

    /// Returns true if the specified program ID bytes is included in the configured program list
    pub fn is_program_selected(&self, program: &[u8]) -> bool {
        self.select_all_programs || self.programs.contains(program)
    }

    /// Returns true if the specified program ID string is included in the configured program list
    pub fn is_program_selected_string(&self, program: &str) -> bool {
        let vec = bs58::decode(program).into_vec().unwrap_or_default();
        self.select_all_programs || self.programs.contains(&vec)
    }

    /// Return true if one or more programs are configured
    pub fn is_enabled(&self) -> bool {
        self.select_all_programs || !self.programs.is_empty()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_create_programs_selector() {
        let programs_selector =
            ProgramsSelector::new(&["9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin".to_string()]);

        assert_eq!(programs_selector.programs.len(), 1);
    }

    #[test]
    fn test_create_programs_selector_multiple_ids() {
        let programs_selector = ProgramsSelector::new(&[
            "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin".to_string(),
            "1xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin".to_string(),
        ]);

        assert_eq!(programs_selector.programs.len(), 2);
        assert!(
            programs_selector
                .is_program_selected_string("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin")
        );
        assert!(
            programs_selector
                .is_program_selected_string("1xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin")
        );
    }

    #[test]
    fn test_programs_selector_all() {
        let programs_selector = ProgramsSelector::new(&["*".to_string()]);

        assert!(programs_selector.select_all_programs);
    }
}
