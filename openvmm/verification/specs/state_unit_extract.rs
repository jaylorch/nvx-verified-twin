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
}
