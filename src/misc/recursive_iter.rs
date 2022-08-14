use smallvec::SmallVec;
use std::collections::VecDeque;

/// Helper to help implement recursive function iteratively.
///
/// In this receipe, recursive calls form a tree (named invocation tree), where each node is a direct call to the recursive function
/// (children of the node will be recursive calls from the direct call).
///
/// ```ignore
/// 1. tree = InvTree::new(<roots of the tree>);
/// 2. while let Some(inv) = tree.next():     # pop the current node from stack
/// 3.     match inv.state:
/// 4.         InvState::Entering(entering_state):
/// 5.             # do something with the data and add next calls if needed
/// 6.             tree.add_recur_invocations(&inv, exiting_state, return_ids, next_ids);
/// 7.         InvState::Exiting(exiting_state):
/// 8.             # handling exiting state
/// ```
pub struct InvTree<U, V>
where
    U: std::fmt::Debug,
    V: std::fmt::Debug,
{
    pub stack: SmallVec<[InvNode<U, V>; 7]>,
}

pub struct InvNode<U, V>
where
    U: std::fmt::Debug,
    V: std::fmt::Debug,
{
    pub parent_id: Option<usize>,
    pub return_id: usize,
    pub state: InvState<U, V>,
}

/// Representing a state in the stack.
#[derive(Debug)]
pub enum InvState<U, V>
where
    U: std::fmt::Debug,
    V: std::fmt::Debug,
{
    Entering(U),
    Exiting(V),
}

#[derive(Debug)]
pub struct InvExitingSeqState<N>
where
    N: std::fmt::Debug,
{
    seq: VecDeque<N>,
    pub n_consumed: usize,
}

pub struct RecurInvocationBuilder<U> {
    pub return_ids: Vec<usize>,
    pub invocations: Vec<U>,
}

impl<U, V> InvTree<U, V>
where
    U: std::fmt::Debug,
    V: std::fmt::Debug,
{
    pub fn new(calls: Vec<U>) -> Self {
        InvTree {
            stack: calls
                .into_iter()
                .rev()
                .map(|call| InvNode {
                    state: InvState::Entering(call),
                    parent_id: None,
                    return_id: 0,
                })
                .collect(),
        }
    }

    pub fn get_mut_parent_state(&mut self, parent_id: usize) -> &mut V {
        // let x = { self as *const InvTree<U, V> as usize };
        match &mut (self.stack[parent_id].state) {
            InvState::Exiting(v) => v,
            _ => {
                // let debug_info =
                //     unsafe { (x as *const InvTree<U, V>).as_ref().unwrap().debug_info() };
                // panic!("the node ({}) you are trying to access is not in correct state. Perhaps you haven't visited it yet?\nDebug info:\n{}", parent_id, debug_info);
                panic!("the node ({}) you are trying to access is not in correct state. Perhaps you haven't visited it yet?", parent_id);
            }
        }
    }

    // Move to the next node in the invocation tree.
    pub fn next(&mut self) -> Option<InvNode<U, V>> {
        self.stack.pop()
    }

    // Add recursive invocation
    pub fn add_recur_invocations(
        &mut self,
        inv: &InvNode<U, V>,
        exiting_state: V,
        return_ids: Vec<usize>,
        next_invs: Vec<U>,
    ) {
        self.stack.reserve(next_invs.len() + 1);

        self.stack.push(InvNode {
            parent_id: inv.parent_id.clone(),
            return_id: inv.return_id,
            state: InvState::Exiting(exiting_state),
        });
        let parent_id = self.stack.len() - 1;
        for (i, inv) in next_invs.into_iter().enumerate().rev() {
            self.stack.push(InvNode {
                parent_id: Some(parent_id),
                return_id: return_ids[i],
                state: InvState::Entering(inv),
            });
        }
    }

    pub fn debug_info(&self) -> String {
        let mut buf = Vec::new();
        buf.push(format!("\n-Stack size: {}", self.stack.len()));
        for (i, inv) in self.stack.iter().enumerate() {
            buf.push(format!("\n- Index {}", i));
            buf.push(format!("\n\t- Parent Id: {:?}", inv.parent_id));
            buf.push(format!("\n\t- Return Id: {}", inv.return_id));
            match &inv.state {
                InvState::Entering(_val) => {
                    buf.push(format!("\n\t- State::Entering"));
                }
                InvState::Exiting(val) => {
                    buf.push(format!("\n\t- State::Exiting({:?})", val));
                }
            }
        }

        buf.join("")
    }
}

impl<U, V> InvState<U, V>
where
    U: std::fmt::Debug,
    V: std::fmt::Debug,
{
    pub fn is_entering(&self) -> bool {
        match self {
            InvState::Entering(_) => true,
            _ => false,
        }
    }
}

impl<U> InvExitingSeqState<U>
where
    U: std::fmt::Debug,
{
    pub fn new() -> Self {
        InvExitingSeqState {
            seq: VecDeque::new(),
            n_consumed: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, value: U) {
        self.seq.push_back(value);
    }

    #[inline(always)]
    pub fn len(&mut self) -> usize {
        self.seq.len()
    }

    #[inline(always)]
    pub fn pop(&mut self) -> U {
        self.n_consumed += 1;
        self.seq.pop_back().unwrap()
    }

    #[inline(always)]
    pub fn consume(self) -> VecDeque<U> {
        self.seq
    }
}

impl<U> RecurInvocationBuilder<U>
where
    U: std::fmt::Debug,
{
    pub fn new() -> Self {
        RecurInvocationBuilder {
            return_ids: Vec::new(),
            invocations: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn len(&mut self) -> usize {
        self.invocations.len()
    }

    #[inline(always)]
    pub fn push(&mut self, return_id: usize, inv: U) {
        self.return_ids.push(return_id);
        self.invocations.push(inv);
    }
}
