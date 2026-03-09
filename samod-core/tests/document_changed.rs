use automerge::{AutomergeError, ROOT, transaction::Transactable};
use samod_core::ChangeOrigin;

use samod_test_harness::{Network, RunningDocIds};

#[test]
fn document_changed_on_receipt_of_message() {
    let mut network = Network::new();

    let alice = network.create_samod("alice");
    let bob = network.create_samod("bob");

    network.connect(alice, bob);

    let RunningDocIds {
        doc_id,
        actor_id: alice_actor,
    } = network.samod(&alice).create_document();

    network.run_until_quiescent();

    let bob_actor = network.samod(&bob).find_document(&doc_id).unwrap();

    network.samod(&bob).pop_doc_changed(bob_actor);

    // Now make a change on alice
    let new_heads = network
        .samod(&alice)
        .with_document_by_actor(alice_actor, |doc| {
            doc.transact::<_, _, AutomergeError>(|tx| {
                tx.put(ROOT, "foo", "bar")?;
                Ok(())
            })
            .unwrap();
            doc.get_heads()
        })
        .unwrap();

    network.run_until_quiescent();

    let change_events = network.samod(&bob).pop_doc_changed(bob_actor);

    assert_eq!(change_events.len(), 1);
    assert_eq!(change_events[0].new_heads, new_heads);
    assert_ne!(change_events[0].old_heads, change_events[0].new_heads);
    assert!(matches!(
        change_events[0].origin,
        ChangeOrigin::Remote { .. }
    ));
}

#[test]
fn document_changed_after_local_change() {
    let mut network = Network::new();

    let alice = network.create_samod("alice");

    let RunningDocIds {
        doc_id: _,
        actor_id: alice_actor,
    } = network.samod(&alice).create_document();

    // Now make a change on alice
    let new_heads = network
        .samod(&alice)
        .with_document_by_actor(alice_actor, |doc| {
            doc.transact::<_, _, AutomergeError>(|tx| {
                tx.put(ROOT, "foo", "bar")?;
                Ok(())
            })
            .unwrap();
            doc.get_heads()
        })
        .unwrap();

    network.run_until_quiescent();

    let change_events = network.samod(&alice).pop_doc_changed(alice_actor);

    assert_eq!(change_events.len(), 1);
    assert_eq!(change_events[0].new_heads, new_heads);
    assert_ne!(change_events[0].old_heads, change_events[0].new_heads);
    assert_eq!(change_events[0].origin, ChangeOrigin::Local);
}

#[test]
fn document_changed_includes_bootstrap_event_when_doc_becomes_ready() {
    let mut network = Network::new();

    let alice = network.create_samod("alice");
    let bob = network.create_samod("bob");

    network.connect(alice, bob);

    let RunningDocIds {
        doc_id,
        actor_id: _alice_actor,
    } = network.samod(&alice).create_document();

    network.run_until_quiescent();

    let bob_actor = network.samod(&bob).find_document(&doc_id).unwrap();

    network.run_until_quiescent();

    let change_events = network.samod(&bob).pop_doc_changed(bob_actor);
    assert!(
        change_events
            .iter()
            .any(|event| event.origin == ChangeOrigin::Bootstrap
                && event.old_heads == event.new_heads)
    );
}
