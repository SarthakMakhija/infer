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
    loop_depth: usize,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            current_function: None,
            loop_depth: 0,
        }
    }

    pub(crate) fn entered_function(&mut self, function: FunctionMetadata) {
        self.current_function = Some(function);
    }

    pub(crate) fn current_function(&self) -> Option<&FunctionMetadata> {
        self.current_function.as_ref()
    }

    pub(crate) fn entered_loop(&mut self) {
        self.loop_depth += 1;
    }

    pub(crate) fn exited_loop(&mut self) {
        self.loop_depth = self.loop_depth.saturating_sub(1);
    }

    pub(crate) fn is_in_loop(&self) -> bool {
        self.loop_depth != 0
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
        state.entered_function(FunctionMetadata::new("calculate".to_string(), true));

        let current = state.current_function().unwrap();
        assert_eq!(current.name, "calculate");
        assert!(current.has_return_type);
    }

    #[test]
    fn state_starts_with_loop_depth_zero() {
        let state = State::new();
        assert!(!state.is_in_loop());
    }

    #[test]
    fn state_tracks_loop_entry_exist() {
        let mut state = State::new();

        state.entered_loop();
        state.exited_loop();
        assert!(!state.is_in_loop());
    }

    #[test]
    fn state_tracks_loop_nesting() {
        let mut state = State::new();

        state.entered_loop();
        assert!(state.is_in_loop());

        state.entered_loop();
        assert!(state.is_in_loop());

        state.exited_loop();
        assert!(state.is_in_loop());

        state.exited_loop();
        assert!(!state.is_in_loop());
    }
}
