pub(crate) struct FunctionMetadata {
    name: String,
    pub(crate) has_return_type: bool,
}

impl FunctionMetadata {
    pub(crate) fn new(name: String, has_return_type: bool) -> Self {
        Self {
            name,
            has_return_type,
        }
    }
}

pub(crate) struct State {
    current_function: Option<FunctionMetadata>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            current_function: None,
        }
    }

    pub(crate) fn in_function(&mut self, function: FunctionMetadata) {
        self.current_function = Some(function);
    }

    pub(crate) fn current_function(&self) -> Option<&FunctionMetadata> {
        self.current_function.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_starts_with_no_current_function() {
        let state = State::new();
        assert!(state.current_function().is_none());
    }

    #[test]
    fn state_records_the_current_function_metadata() {
        let mut state = State::new();
        state.in_function(FunctionMetadata::new("calculate".to_string(), true));

        let current = state.current_function().unwrap();
        assert_eq!(current.name, "calculate");
        assert!(current.has_return_type);
    }
}
