// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::sync::Arc;
use vstd::prelude::*;

verus! {
    /// Names of every failed item, preserving the input order and duplicates.
    pub open spec fn error_names<T, E>(
        input: Seq<(Arc<str>, Result<T, E>)>,
    ) -> Seq<Arc<str>> {
        input.fold_left(Seq::empty(), |names: Seq<Arc<str>>, item: (Arc<str>, Result<T, E>)|
            match item.1 {
                Ok(_) => names,
                Err(_) => names.push(item.0),
            })
    }

    pub proof fn error_names_push<T, E>(
        input: Seq<(Arc<str>, Result<T, E>)>,
        item: (Arc<str>, Result<T, E>),
    )
        ensures
            error_names(input.push(item)) ==
                match item.1 {
                    Ok(_) => error_names(input),
                    Err(_) => error_names(input).push(item.0),
                },
    {
        reveal_with_fuel(Seq::fold_left, 2);
        assert(input.push(item).drop_last() =~= input);
        assert(input.push(item).last() == item);
    }

    /// Error contents are deliberately opaque; only the original names matter.
    pub open spec fn collected_names(
        errors: Seq<(Arc<str>, anyhow::Error)>,
    ) -> Seq<Arc<str>> {
        errors.map(|_: int, pair: (Arc<str>, anyhow::Error)| pair.0)
    }

    pub open spec fn filtered_choices<U>(choices: Seq<Option<U>>) -> Seq<U> {
        choices.filter_map(|choice: Option<U>| choice)
    }

    pub proof fn filtered_choices_push<U>(choices: Seq<Option<U>>, choice: Option<U>)
        ensures
            filtered_choices(choices.push(choice)) ==
                match choice {
                    Some(value) => filtered_choices(choices).push(value),
                    None => filtered_choices(choices),
                },
    {
        reveal_with_fuel(Seq::filter_map, 2);
        assert(choices.push(choice).drop_last() =~= choices);
        assert(choices.push(choice).last() == choice);
        if let Some(value) = choice {
            assert(filtered_choices(choices) + seq![value] =~=
                filtered_choices(choices).push(value));
        }
    }

    pub open spec fn choices_valid<T, E, U, F>(
        input: Seq<(Arc<str>, Result<T, E>)>,
        f: F,
        choices: Seq<Option<U>>,
    ) -> bool
        where F: Fn(Arc<str>, T) -> Option<U>,
    {
        choices.len() == input.len()
        && forall |i: int| 0 <= i < input.len() ==> match input[i].1 {
            Ok(value) => f.ensures((input[i].0, value), #[trigger] choices[i]),
            Err(_) => #[trigger] choices[i] is None,
        }
    }

    pub open spec fn outputs_allowed<T, E, U, F>(
        input: Seq<(Arc<str>, Result<T, E>)>,
        f: F,
        outputs: Seq<U>,
    ) -> bool
        where F: Fn(Arc<str>, T) -> Option<U>,
    {
        exists |choices: Seq<Option<U>>| #![auto]
            choices_valid(input, f, choices) && outputs == filtered_choices(choices)
    }

    #[verifier::prophetic]
    pub(super) open spec fn check_post<E, I>(
        op: &'static str,
        iter: I,
        ret: Result<(), super::StateTransitionError>,
    ) -> bool
        where I: Iterator<Item = (Arc<str>, Result<(), E>)> + super::ExtractIteratorSpec,
    {
        &&& ret.is_ok() == (error_names(iter.remaining()).len() == 0)
        &&& match ret {
            Ok(_) => true,
            Err(ref failure) =>
                failure.op == op
                && collected_names(failure.errors.0@) == error_names(iter.remaining()),
        }
    }

    /// Exercises the production loop with vstd's Vec iterator and a real Fn.
    fn vec_fn_witness(
        op: &'static str,
        input: Vec<(Arc<str>, Result<(), anyhow::Error>)>,
    ) -> (ret: Result<Vec<()>, super::StateTransitionError>)
        ensures
            ret.is_ok() == (error_names(input@).len() == 0),
            match ret {
                Ok(_) => true,
                Err(ref failure) =>
                    failure.op == op && collected_names(failure.errors.0@) == error_names(input@),
            },
    {
        super::extract(op, input.into_iter(), |_, _| Some(()))
    }

    fn ordered_errors_after_success_witness(
        first_name: Arc<str>,
        middle_name: Arc<str>,
        last_name: Arc<str>,
        first_error: anyhow::Error,
        last_error: anyhow::Error,
    ) -> (ret: Result<Vec<()>, super::StateTransitionError>)
        ensures
            ret matches Err(ref failure)
                && failure.op == "save"
                && collected_names(failure.errors.0@) == seq![first_name, last_name],
    {
        let mut input = Vec::new();
        input.push((first_name, Err(first_error)));
        input.push((middle_name, Ok(())));
        input.push((last_name, Err(last_error)));
        proof {
            assert(input@ =~= seq![
                (first_name, Err(first_error)),
                (middle_name, Ok(())),
                (last_name, Err(last_error)),
            ]);
            assert(error_names(seq![
                (first_name, Err(first_error)),
                (middle_name, Ok(())),
                (last_name, Err(last_error)),
            ]) == seq![first_name, last_name]) by (compute);
        }
        vec_fn_witness("save", input)
    }

    fn duplicate_error_name_witness(
        name: Arc<str>,
        first_error: anyhow::Error,
        second_error: anyhow::Error,
    ) -> (ret: Result<Vec<()>, super::StateTransitionError>)
        ensures
            ret matches Err(ref failure)
                && failure.op == "save"
                && collected_names(failure.errors.0@) == seq![name, name],
    {
        let mut input = Vec::new();
        let repeated = name.clone();
        input.push((name, Err(first_error)));
        input.push((repeated, Err(second_error)));
        proof {
            assert(input@ =~= seq![(name, Err(first_error)), (repeated, Err(second_error))]);
            assert(error_names::<(), anyhow::Error>(seq![
                (name, Err(first_error)),
                (repeated, Err(second_error)),
            ]) == seq![name, repeated]) by (compute);
        }
        vec_fn_witness("save", input)
    }

    fn check_empty_witness() -> (ret: Result<(), super::StateTransitionError>)
        ensures ret is Ok,
    {
        let input: Vec<(Arc<str>, Result<(), anyhow::Error>)> = Vec::new();
        super::check("reset", input.into_iter())
    }

    fn check_ordered_error_witness(
        first_name: Arc<str>,
        middle_name: Arc<str>,
        last_name: Arc<str>,
        first_error: anyhow::Error,
        last_error: anyhow::Error,
    ) -> (ret: Result<(), super::StateTransitionError>)
        ensures
            ret matches Err(ref failure)
                && failure.op == "restore"
                && collected_names(failure.errors.0@) == seq![first_name, last_name],
    {
        let mut input = Vec::new();
        input.push((first_name, Err(first_error)));
        input.push((middle_name, Ok(())));
        input.push((last_name, Err(last_error)));
        proof {
            assert(input@ =~= seq![
                (first_name, Err(first_error)),
                (middle_name, Ok(())),
                (last_name, Err(last_error)),
            ]);
            assert(error_names(seq![
                (first_name, Err(first_error)),
                (middle_name, Ok(())),
                (last_name, Err(last_error)),
            ]) == seq![first_name, last_name]) by (compute);
        }
        super::check("restore", input.into_iter())
    }

    fn check_duplicate_error_name_witness(
        name: Arc<str>,
        first_error: anyhow::Error,
        second_error: anyhow::Error,
    ) -> (ret: Result<(), super::StateTransitionError>)
        ensures
            ret matches Err(ref failure)
                && failure.op == "reset"
                && collected_names(failure.errors.0@) == seq![name, name],
    {
        let mut input = Vec::new();
        let repeated = name.clone();
        input.push((name, Err(first_error)));
        input.push((repeated, Err(second_error)));
        proof {
            assert(input@ =~= seq![(name, Err(first_error)), (repeated, Err(second_error))]);
            assert(error_names::<(), anyhow::Error>(seq![
                (name, Err(first_error)),
                (repeated, Err(second_error)),
            ]) == seq![name, repeated]) by (compute);
        }
        super::check("reset", input.into_iter())
    }

    fn empty_output_witness() -> (ret: Result<Vec<u64>, super::StateTransitionError>)
        ensures ret matches Ok(ref values) && values@ == Seq::<u64>::empty(),
    {
        let input: Vec<(Arc<str>, Result<u64, anyhow::Error>)> = Vec::new();
        let f = |_: Arc<str>, value: u64| -> (out: Option<u64>)
            ensures out == if value % 2 == 0 { Some(value) } else { None },
        {
            if value % 2 == 0 { Some(value) } else { None }
        };
        super::extract("save", input.into_iter(), f)
    }

    fn some_none_output_witness(
        odd_name: Arc<str>,
        even_name: Arc<str>,
    ) -> (ret: Result<Vec<u64>, super::StateTransitionError>)
        ensures ret matches Ok(ref values) && values@ == seq![4u64],
    {
        let mut input: Vec<(Arc<str>, Result<u64, anyhow::Error>)> = Vec::new();
        input.push((odd_name, Ok(3)));
        input.push((even_name, Ok(4)));
        let ghost initial = input@;
        let f = |_: Arc<str>, value: u64| -> (out: Option<u64>)
            ensures out == if value % 2 == 0 { Some(value) } else { None },
        {
            if value % 2 == 0 { Some(value) } else { None }
        };
        let ret = super::extract("save", input.into_iter(), f);
        proof {
            assert(initial =~= seq![(odd_name, Ok(3u64)), (even_name, Ok(4u64))]);
            assert(error_names::<u64, anyhow::Error>(
                seq![(odd_name, Ok(3u64)), (even_name, Ok(4u64))])
                == Seq::<Arc<str>>::empty()) by (compute);
            if let Ok(ref values) = ret {
                let ghost choices = choose |choices: Seq<Option<u64>>| #![auto]
                    choices_valid(initial, f, choices) && values@ == filtered_choices(choices);
                assert(choices[0] == None);
                assert(choices[1] == Some(4u64));
                assert(choices =~= seq![None, Some(4u64)]);
                assert(filtered_choices(seq![None, Some(4u64)]) == seq![4u64]) by (compute);
                assert(values@ == seq![4u64]);
            }
        }
        ret
    }

    fn ordered_values_witness(
        first_name: Arc<str>,
        skipped_name: Arc<str>,
        last_name: Arc<str>,
    ) -> (ret: Result<Vec<u64>, super::StateTransitionError>)
        ensures ret matches Ok(ref values) && values@ == seq![8u64, 6u64],
    {
        let mut input: Vec<(Arc<str>, Result<u64, anyhow::Error>)> = Vec::new();
        input.push((first_name, Ok(8)));
        input.push((skipped_name, Ok(5)));
        input.push((last_name, Ok(6)));
        let ghost initial = input@;
        let f = |_: Arc<str>, value: u64| -> (out: Option<u64>)
            ensures out == if value % 2 == 0 { Some(value) } else { None },
        {
            if value % 2 == 0 { Some(value) } else { None }
        };
        let ret = super::extract("save", input.into_iter(), f);
        proof {
            assert(initial =~= seq![
                (first_name, Ok(8u64)),
                (skipped_name, Ok(5u64)),
                (last_name, Ok(6u64)),
            ]);
            assert(error_names::<u64, anyhow::Error>(seq![
                (first_name, Ok(8u64)),
                (skipped_name, Ok(5u64)),
                (last_name, Ok(6u64)),
            ]) == Seq::<Arc<str>>::empty()) by (compute);
            if let Ok(ref values) = ret {
                let ghost choices = choose |choices: Seq<Option<u64>>| #![auto]
                    choices_valid(initial, f, choices) && values@ == filtered_choices(choices);
                assert(choices[0] == Some(8u64));
                assert(choices[1] == None);
                assert(choices[2] == Some(6u64));
                assert(choices =~= seq![Some(8u64), None, Some(6u64)]);
                assert(filtered_choices(seq![Some(8u64), None, Some(6u64)])
                    == seq![8u64, 6u64]) by (compute);
                assert(values@ == seq![8u64, 6u64]);
            }
        }
        ret
    }
}
