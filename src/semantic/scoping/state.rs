use crate::ast::statement::NodeId;
use crate::semantic::SymbolId;
use std::collections::HashMap;

/// Represents a deferred function call that will be resolved at the end of the pass.
///
/// Forward references to functions are permitted, so any call to a function
/// not yet declared is collected as a pending call and validated after the program walk.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PendingCall {
    pub(crate) name: String,
    pub(crate) argument_count: usize,
    pub(crate) callee_node_id: NodeId,
}

/// Metadata storing the signature details of a declared function.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FunctionMetadata {
    pub(crate) name: String,
    pub(crate) parameter_types: Vec<Option<String>>,
    pub(crate) return_type: Option<String>,
    pub(crate) parameter_symbols: Vec<SymbolId>,
}

impl FunctionMetadata {
    /// Creates a new `FunctionMetadata` containing signature information.
    pub(crate) fn new(
        name: String,
        parameter_types: Vec<Option<String>>,
        return_type: Option<String>,
    ) -> Self {
        Self {
            name,
            parameter_types,
            return_type,
            parameter_symbols: vec![],
        }
    }

    /// Builder helper to associate parameter SymbolIds with the function metadata.
    pub(crate) fn with_symbols(mut self, parameter_symbols: Vec<SymbolId>) -> Self {
        self.parameter_symbols = parameter_symbols;
        self
    }

    /// Returns the number of parameters the function accepts.
    pub(crate) fn parameter_count(&self) -> usize {
        self.parameter_types.len()
    }

    /// Returns true if the function has an annotated return type.
    pub(crate) fn has_return_type(&self) -> bool {
        self.return_type.is_some()
    }
}

/// The compiler semantic state tracked during the symbol resolution walk.
pub(crate) struct State {
    current_function: Option<FunctionMetadata>,
    global_functions: HashMap<SymbolId, FunctionMetadata>,
    pub(crate) pending_calls: Vec<PendingCall>,
    loop_depth: usize,
    encountered_break: bool,
    encountered_return: bool,
}

impl State {
    /// Creates a new, empty semantic `State`.
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

    /// Defers a function call for later arity and declaration validation.
    pub(crate) fn add_pending_call(
        &mut self,
        name: String,
        argument_count: usize,
        callee_node_id: NodeId,
    ) {
        self.pending_calls.push(PendingCall {
            name,
            argument_count,
            callee_node_id,
        });
    }

    /// Registers a global function declaration and marks it as the current active function.
    pub(crate) fn add_global_function(&mut self, symbol_id: SymbolId, function: FunctionMetadata) {
        self.global_functions.insert(symbol_id, function.clone());
        self.current_function = Some(function);
    }

    /// Retrieves metadata for a previously declared global function.
    pub(crate) fn get_global_function(&self, symbol_id: &SymbolId) -> Option<&FunctionMetadata> {
        self.global_functions.get(symbol_id)
    }

    /// Resets the current function context when exiting a function body.
    pub(crate) fn exited_function(&mut self) {
        self.current_function = None;
    }

    /// Returns the metadata of the currently active function, if any.
    pub(crate) fn current_function(&self) -> Option<&FunctionMetadata> {
        self.current_function.as_ref()
    }

    /// Enters a loop statement context.
    pub(crate) fn entered_loop(&mut self) {
        self.loop_depth += 1;
    }

    /// Exits a loop statement context.
    pub(crate) fn exited_loop(&mut self) {
        self.loop_depth = self.loop_depth.saturating_sub(1);
    }

    /// Returns `true` if currently within one or more loop contexts.
    pub(crate) fn is_in_loop(&self) -> bool {
        self.loop_depth != 0
    }

    /// Records that a break control flow statement was encountered.
    pub(crate) fn encountered_break(&mut self) {
        self.encountered_break = true;
    }

    /// Records that a function return statement was encountered.
    pub(crate) fn encountered_return(&mut self) {
        self.encountered_return = true;
    }

    /// Resets the break detection flag.
    pub(crate) fn reset_break(&mut self) {
        self.encountered_break = false;
    }

    /// Resets the return detection flag.
    pub(crate) fn reset_return(&mut self) {
        self.encountered_return = false;
    }

    /// Returns `true` if subsequent statements in the block are unreachable.
    pub(crate) fn is_unreachable(&self) -> bool {
        self.encountered_break || self.encountered_return
    }

    /// Returns a reference to the map of declared global functions and their metadata.
    pub(crate) fn global_functions(&self) -> &HashMap<SymbolId, FunctionMetadata> {
        &self.global_functions
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
            FunctionMetadata::new("calculate".to_string(), vec![], Some("i32".to_string())),
        );

        let current = state.get_global_function(&SymbolId(0)).unwrap();
        assert_eq!(current.name, "calculate");
    }

    #[test]
    fn state_records_the_current_function_metadata() {
        let mut state = State::new();
        state.add_global_function(
            SymbolId(0),
            FunctionMetadata::new("calculate".to_string(), vec![], Some("i32".to_string())),
        );

        let function_metadata = state.global_functions.get(&SymbolId(0)).unwrap();
        assert_eq!(function_metadata.name, "calculate");
    }

    #[test]
    fn state_exits_the_function() {
        let mut state = State::new();
        state.add_global_function(
            SymbolId(0),
            FunctionMetadata::new("calculate".to_string(), vec![], Some("i32".to_string())),
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

#[cfg(test)]
mod function_metadata_tests {
    use super::*;

    #[test]
    fn parameter_count() {
        let metadata =
            FunctionMetadata::new("calculate".to_string(), vec![Some("i32".to_string())], None);
        assert_eq!(metadata.parameter_count(), 1);
    }

    #[test]
    fn has_return_type() {
        let metadata =
            FunctionMetadata::new("calculate".to_string(), vec![], Some("bool".to_string()));
        assert!(metadata.has_return_type());
    }

    #[test]
    fn does_not_have_return_type() {
        let metadata = FunctionMetadata::new("calculate".to_string(), vec![], None);
        assert!(!metadata.has_return_type());
    }
}
