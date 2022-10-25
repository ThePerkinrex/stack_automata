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

pub trait Movement<'a, 'b, VocabElement, StackData, Q>
where
    'a: 'b,
{
    fn f(
        &'a self,
        state: &Q,
        v: &Option<VocabElement>,
        s: &StackData,
    ) -> Option<&'b (Q, Vec<StackData>)>;
}

pub type Movements<VocabElement, StackData, Q> =
    HashMap<(Q, Option<VocabElement>, StackData), (Q, Vec<StackData>)>;

impl<'a, 'b, VocabElement, StackData, Q> Movement<'a, 'b, VocabElement, StackData, Q>
    for Movements<VocabElement, StackData, Q>
where
    (Q, Option<VocabElement>, StackData): Hash + Eq,
    StackData: Clone,
    Q: Clone,
    VocabElement: Clone,
    'a: 'b,
{
    fn f(
        &'a self,
        state: &Q,
        v: &Option<VocabElement>,
        s: &StackData,
    ) -> Option<&'b (Q, Vec<StackData>)> {
        self.get(&(state.clone(), v.clone(), s.clone()))
    }
}

impl<'a, 'b, VocabElement, StackData, Q, F> Movement<'a, 'b, VocabElement, StackData, Q> for F
where
    F: Fn(&Q, &Option<VocabElement>, &StackData) -> Option<&'b (Q, Vec<StackData>)> + 'a,
    StackData: 'b,
    Q: 'b,
    'a: 'b,
{
    fn f(
        &'a self,
        state: &Q,
        v: &Option<VocabElement>,
        s: &StackData,
    ) -> Option<&'b (Q, Vec<StackData>)> {
        self(state, v, s)
    }
}

#[derive(Debug, Clone)]
pub struct AutomataBuilder<StackData, Q, M> {
    state: Q,
    stack: Stack<StackData>,
    movements: M,
}

impl<StackData, Q, M> AutomataBuilder<StackData, Q, M> {
    pub fn new<S>(initial_state: Q, initial_stack: S, movements: M) -> Self
    where
        S: Into<Stack<StackData>>,
    {
        Self {
            state: initial_state,
            stack: initial_stack.into(),
            movements,
        }
    }

    pub fn build<'a, 'b, V, W>(&self, word: W) -> Automata<V, StackData, Q, W, M>
    where
        W: Iterator<Item = V>,
        Q: Clone,
        StackData: Clone,
        M: Clone,
        M: Movement<'a, 'b, V, StackData, Q>,
        'a: 'b,
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
pub struct Automata<VocabElement, StackData, Q, Word, M>
where
    Word: Iterator<Item = VocabElement>,
{
    state: Q,
    stack: Stack<StackData>,
    word: Word,
    movements: M,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AutomataResult {
    Accept,
    Processing,
    NotAccepting,
}

impl<VocabElement, StackData, Q, Word, M> Automata<VocabElement, StackData, Q, Word, M>
where
    Word: Iterator<Item = VocabElement>,
{
    pub fn new<S>(word: Word, initial_state: Q, initial_stack: S, movements: M) -> Self
    where
        S: Into<Stack<StackData>>,
    {
        Self {
            state: initial_state,
            stack: initial_stack.into(),
            word,
            movements,
        }
    }

    pub fn run<'a, 'b>(&mut self) -> AutomataResult
    where
        (Q, Option<VocabElement>, StackData): Hash + Eq,
        StackData: Clone,
        Q: Clone,
        M: Movement<'a, 'b, VocabElement, StackData, Q>,
        'a: 'b,
    {
        let v = self.word.next();
        let s = self.stack.pop();
        match (v, s) {
            (None, None) => AutomataResult::Accept,
            (v, Some(s)) => {
                let m = self.movements.f(&self.state, &v, &s).cloned();

                if let Some((state, new_stack)) = m {
                    self.state = state.clone();
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

    pub fn complete<'a, 'b>(mut self) -> bool
    where
        (Q, Option<VocabElement>, StackData): Hash + Eq,
        StackData: Clone,
        Q: Clone,
        M: Movement<'a, 'b, VocabElement, StackData, Q>,
        'a: 'b,
        Self: 'a,
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
