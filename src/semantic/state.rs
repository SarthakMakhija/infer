use crate::semantic::SymbolId;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PendingCall {
    pub(crate) name: String,
    pub(crate) argument_count: usize,
}

#[derive(Clone)]
pub(crate) struct FunctionMetadata {
    pub(crate) name: String,
    pub(crate) parameter_count: usize,
    pub(crate) has_return_type: bool,
}

impl FunctionMetadata {
    pub(crate) fn new(name: String, parameter_count: usize, has_return_type: bool) -> Self {
        Self {
            name,
            parameter_count,
            has_return_type,
        }
    }
}

pub(crate) struct State {
    current_function: Option<FunctionMetadata>,
    global_functions: HashMap<SymbolId, FunctionMetadata>,
    pub(crate) pending_calls: Vec<PendingCall>,
    loop_depth: usize,
    encountered_break: bool,
    encountered_return: bool,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            current_function: None,
            global_functions: HashMap::new(),
            pending_calls: Vec::new(),
            loop_depth: 0,
            encountered_break: false,
            encountered_return: false,
        }
    }

    pub(crate) fn add_pending_call(&mut self, name: String, argument_count: usize) {
        self.pending_calls.push(PendingCall {
            name,
            argument_count,
        });
    }

    pub(crate) fn add_global_function(&mut self, symbol_id: SymbolId, function: FunctionMetadata) {
        self.global_functions.insert(symbol_id, function.clone());
        self.current_function = Some(function);
    }

    pub(crate) fn get_global_function(&self, symbol_id: &SymbolId) -> Option<&FunctionMetadata> {
        self.global_functions.get(symbol_id)
    }

    pub(crate) fn exited_function(&mut self) {
        self.current_function = None;
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

    pub(crate) fn encountered_break(&mut self) {
        self.encountered_break = true;
    }

    pub(crate) fn encountered_return(&mut self) {
        self.encountered_return = true;
    }

    pub(crate) fn reset_break(&mut self) {
        self.encountered_break = false;
    }

    pub(crate) fn reset_return(&mut self) {
        self.encountered_return = false;
    }

    pub(crate) fn is_unreachable(&self) -> bool {
        self.encountered_break || self.encountered_return
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
    fn state_records_the_global_function_metadata() {
        let mut state = State::new();
        state.add_global_function(
            SymbolId(0),
            FunctionMetadata::new("calculate".to_string(), 0, true),
        );

        let current = state.get_global_function(&SymbolId(0)).unwrap();
        assert_eq!(current.name, "calculate");
        assert!(current.has_return_type);
    }

    #[test]
    fn state_records_the_current_function_metadata() {
        let mut state = State::new();
        state.add_global_function(
            SymbolId(0),
            FunctionMetadata::new("calculate".to_string(), 0, true),
        );

        let function_metadata = state.global_functions.get(&SymbolId(0)).unwrap();
        assert_eq!(function_metadata.name, "calculate");
        assert!(function_metadata.has_return_type);
    }

    #[test]
    fn state_exits_the_function() {
        let mut state = State::new();
        state.add_global_function(
            SymbolId(0),
            FunctionMetadata::new("calculate".to_string(), 0, true),
        );
        state.exited_function();

        assert!(state.current_function().is_none());
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

    #[test]
    fn state_starts_with_no_unreachable_flags() {
        let state = State::new();
        assert!(!state.is_unreachable());
    }

    #[test]
    fn state_tracks_when_break_is_encountered() {
        let mut state = State::new();
        state.encountered_break();
        assert!(state.is_unreachable());
    }

    #[test]
    fn state_tracks_when_return_is_encountered() {
        let mut state = State::new();
        state.encountered_return();
        assert!(state.is_unreachable());
    }

    #[test]
    fn state_resets_break_flag_correctly() {
        let mut state = State::new();
        state.encountered_break();
        state.reset_break();
        assert!(!state.is_unreachable());
    }

    #[test]
    fn state_resets_return_flag_correctly() {
        let mut state = State::new();
        state.encountered_return();
        state.reset_return();
        assert!(!state.is_unreachable());
    }
}
