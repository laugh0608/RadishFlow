//! Shared material-unit graph algorithms; strategies own capability checks.
use rf_types::UnitId;
use std::collections::{BTreeMap, BTreeSet};
pub(crate) type Graph = BTreeMap<UnitId, BTreeSet<UnitId>>;
pub(crate) fn topological_order(graph: &Graph) -> Option<Vec<UnitId>> {
    let mut counts: BTreeMap<_, usize> = graph.keys().cloned().map(|id| (id, 0)).collect();
    for children in graph.values() {
        for child in children {
            *counts.get_mut(child)? += 1;
        }
    }
    let mut ready: BTreeSet<_> = counts
        .iter()
        .filter(|(_, n)| **n == 0)
        .map(|(id, _)| id.clone())
        .collect();
    let mut result = Vec::new();
    while let Some(id) = ready.pop_first() {
        for child in &graph[&id] {
            let count = counts.get_mut(child)?;
            *count -= 1;
            if *count == 0 {
                ready.insert(child.clone());
            }
        }
        result.push(id);
    }
    (result.len() == graph.len()).then_some(result)
}

// Iterative Kosaraju traversal avoids a recursion/stack limit on large acyclic inputs.
pub(crate) fn strongly_connected_components(graph: &Graph) -> Vec<Vec<UnitId>> {
    let mut visited = BTreeSet::new();
    let mut finish = Vec::new();
    for root in graph.keys() {
        let mut stack = vec![(root.clone(), false)];
        while let Some((id, exiting)) = stack.pop() {
            if exiting {
                finish.push(id);
                continue;
            }
            if !visited.insert(id.clone()) {
                continue;
            }
            stack.push((id.clone(), true));
            stack.extend(graph[&id].iter().rev().cloned().map(|child| (child, false)));
        }
    }
    let mut reverse: Graph = graph
        .keys()
        .cloned()
        .map(|id| (id, BTreeSet::new()))
        .collect();
    for (source, children) in graph {
        for child in children {
            reverse
                .get_mut(child)
                .expect("graph node")
                .insert(source.clone());
        }
    }
    visited.clear();
    let mut result = Vec::new();
    for root in finish.into_iter().rev() {
        if visited.contains(&root) {
            continue;
        }
        let mut members = Vec::new();
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            if !visited.insert(id.clone()) {
                continue;
            }
            stack.extend(reverse[&id].iter().cloned());
            members.push(id);
        }
        members.sort();
        result.push(members);
    }
    result
}

/// SCC condensation order; dependencies first and stable UnitId ties.
pub(crate) fn ordered_components(graph: &Graph) -> Vec<Vec<UnitId>> {
    let components = strongly_connected_components(graph);
    let membership: BTreeMap<_, _> = components
        .iter()
        .enumerate()
        .flat_map(|(i, members)| members.iter().cloned().map(move |id| (id, i)))
        .collect();
    let mut children = vec![BTreeSet::new(); components.len()];
    for (source, sinks) in graph {
        for sink in sinks {
            let (a, b) = (membership[source], membership[sink]);
            if a != b {
                children[a].insert(b);
            }
        }
    }
    let mut indegree = vec![0; components.len()];
    for sinks in &children {
        for &sink in sinks {
            indegree[sink] += 1;
        }
    }
    let mut ready: BTreeSet<_> = components
        .iter()
        .enumerate()
        .filter(|(i, _)| indegree[*i] == 0)
        .map(|(i, c)| (c[0].clone(), i))
        .collect();
    let mut ordered = Vec::new();
    while let Some((_, i)) = ready.pop_first() {
        ordered.push(components[i].clone());
        for &j in &children[i] {
            indegree[j] -= 1;
            if indegree[j] == 0 {
                ready.insert((components[j][0].clone(), j));
            }
        }
    }
    ordered
}
