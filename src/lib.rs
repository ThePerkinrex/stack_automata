use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Stack<StackData>(Vec<StackData>);

impl<StackData> Stack<StackData> {
    pub fn new(data: Vec<StackData>) -> Self {
        Self(data)
    }

    pub fn push(&mut self, e: StackData) {
        self.0.push(e);
    }

    pub fn pop(&mut self) -> Option<StackData> {
        self.0.pop()
    }
}

impl<F, T> From<F> for Stack<T>
where
    F: Into<Vec<T>>,
{
    fn from(f: F) -> Self {
        Self::new(f.into())
    }
}

pub type Movements<VocabElement, StackData, Q> =
    HashMap<(Q, Option<VocabElement>, StackData), (Q, Vec<StackData>)>;

#[derive(Debug, Clone)]
pub struct AutomataBuilder<VocabElement, StackData, Q> {
    state: Q,
    stack: Stack<StackData>,
    movements: Movements<VocabElement, StackData, Q>,
}

impl<VocabElement, StackData, Q> AutomataBuilder<VocabElement, StackData, Q> {
    pub fn new<S>(
        initial_state: Q,
        initial_stack: S,
        movements: Movements<VocabElement, StackData, Q>,
    ) -> Self
    where
        S: Into<Stack<StackData>>,
    {
        Self {
            state: initial_state,
            stack: initial_stack.into(),
            movements,
        }
    }

    pub fn build<W>(&self, word: W) -> Automata<VocabElement, StackData, Q, W>
    where
        W: Iterator<Item = VocabElement>,
        Q: Clone,
        StackData: Clone,
        Movements<VocabElement, StackData, Q>: Clone,
    {
        Automata::new(
            word,
            self.state.clone(),
            self.stack.clone(),
            self.movements.clone(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Automata<VocabElement, StackData, Q, Word>
where
    Word: Iterator<Item = VocabElement>,
{
    state: Option<Q>,
    stack: Stack<StackData>,
    word: Word,
    movements: Movements<VocabElement, StackData, Q>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AutomataResult {
    Accept,
    Processing,
    NotAccepting,
}

impl<VocabElement, StackData, Q, Word> Automata<VocabElement, StackData, Q, Word>
where
    Word: Iterator<Item = VocabElement>,
{
    pub fn new<S>(
        word: Word,
        initial_state: Q,
        initial_stack: S,
        movements: Movements<VocabElement, StackData, Q>,
    ) -> Self
    where
        S: Into<Stack<StackData>>,
    {
        Self {
            state: Some(initial_state),
            stack: initial_stack.into(),
            word,
            movements,
        }
    }

    pub fn run(&mut self) -> AutomataResult
    where
        (Q, Option<VocabElement>, StackData): Hash + Eq,
        StackData: Clone,
        Q: Clone,
    {
        let v = self.word.next();
        let s = self.stack.pop();
        match (v, s) {
            (None, None) => AutomataResult::Accept,
            (v, Some(s)) => {
                let m = self.movements.get(&(self.state.take().unwrap(), v, s));

                if let Some((state, new_stack)) = m {
                    self.state = Some(state.clone());
                    for elem in new_stack.iter().rev() {
                        self.stack.push(elem.clone());
                    }
                    AutomataResult::Processing
                } else {
                    AutomataResult::NotAccepting
                }
            }
            _ => AutomataResult::NotAccepting,
        }
    }

    pub fn complete(mut self) -> bool
    where
        (Q, Option<VocabElement>, StackData): Hash + Eq,
        StackData: Clone,
        Q: Clone,
    {
        let mut r = AutomataResult::Processing;
        while r == AutomataResult::Processing {
            r = self.run();
        }
        r == AutomataResult::Accept
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{AutomataBuilder, Movements};

    #[test]
    /// Test for (a^n)(b^n) where n >= 1
    /// V1
    fn test_an_bn_n_ge_1_v1() {
        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        enum State {
            Q0,
            Q1,
        }

        use State::*;

        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        enum StackElement {
            A0,
            A,
        }

        use StackElement::*;

        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        enum Vocab {
            a,
            b,
        }
        use Vocab::*;

        let mut ruleset: Movements<Vocab, StackElement, State> = HashMap::new();
        ruleset.insert((Q0, Some(a), A0), (Q0, vec![A]));
        ruleset.insert((Q0, Some(a), A), (Q0, vec![A, A]));
        ruleset.insert((Q0, Some(b), A), (Q1, vec![]));
        ruleset.insert((Q1, Some(b), A), (Q1, vec![]));

        let automata_builder = AutomataBuilder::new(Q0, vec![A0], ruleset);
        assert!(automata_builder.build([a, b].into_iter()).complete());
        assert!(automata_builder.build([a, a, b, b].into_iter()).complete());
        assert!(automata_builder
            .build([a, a, a, b, b, b].into_iter())
            .complete());
        assert!(!automata_builder.build([a].into_iter()).complete());
        assert!(!automata_builder.build([b].into_iter()).complete());
        assert!(!automata_builder.build([].into_iter()).complete());
    }

    #[test]
    /// Test for (a^n)(b^n) where n >= 1
    /// V2
    fn test_an_bn_n_ge_1_v2() {
        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        enum State {
            Q0,
            Q1,
        }

        use State::*;

        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        enum StackElement {
            A0,
            A,
        }

        use StackElement::*;

        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        enum Vocab {
            a,
            b,
        }
        use Vocab::*;

        let mut ruleset: Movements<Vocab, StackElement, State> = HashMap::new();
        ruleset.insert((Q0, Some(a), A0), (Q0, vec![A, A0]));
        ruleset.insert((Q0, Some(a), A), (Q0, vec![A, A]));
        ruleset.insert((Q0, Some(b), A), (Q1, vec![]));
        ruleset.insert((Q1, Some(b), A), (Q1, vec![]));
        ruleset.insert((Q1, None, A0), (Q1, vec![])); // Accept

        let automata_builder = AutomataBuilder::new(Q0, vec![A0], ruleset);
        assert!(automata_builder.build([a, b].into_iter()).complete());
        assert!(automata_builder.build([a, a, b, b].into_iter()).complete());
        assert!(automata_builder
            .build([a, a, a, b, b, b].into_iter())
            .complete());
        assert!(!automata_builder.build([a].into_iter()).complete());
        assert!(!automata_builder.build([b].into_iter()).complete());
        assert!(!automata_builder.build([].into_iter()).complete());
    }
}
