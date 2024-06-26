use std::borrow::Cow;

use fuel_core_storage::test_helpers::MockStorage;
use fuel_core_types::entities::message::Message;
use test_case::test_case;

use super::*;

#[test]
fn test_insert_messages() {
    let mut db = MockStorage::default();
    db.expect_insert::<Messages>()
        .times(2)
        .returning(|_, _| Ok(None));
    db.expect_insert::<RelayerMetadata>()
        .times(1)
        .withf(|_, v| **v == 12)
        .returning(|_, _| Ok(None));
    db.expect_commit().returning(|| Ok(()));
    db.expect_get::<RelayerMetadata>()
        .once()
        .returning(|_| Ok(Some(std::borrow::Cow::Owned(9u64.into()))));
    let mut db = db.into_transactional();

    let m = Message {
        amount: 10,
        da_height: 12u64.into(),
        ..Default::default()
    };
    let mut m2 = m.clone();
    m2.nonce = 1.into();
    assert_ne!(m.id(), m2.id());
    let messages = [m, m2];
    db.insert_messages(&12u64.into(), &messages[..]).unwrap();
}

#[test]
fn insert_always_raises_da_height_monotonically() {
    let messages: Vec<_> = (0..10)
        .map(|i| Message {
            amount: i,
            da_height: i.into(),
            ..Default::default()
        })
        .collect();

    let mut db = MockStorage::default();
    db.expect_insert::<Messages>().returning(|_, _| Ok(None));
    db.expect_insert::<RelayerMetadata>()
        .once()
        .withf(|_, v| **v == 9)
        .returning(|_, _| Ok(None));
    db.expect_commit().returning(|| Ok(()));
    db.expect_get::<RelayerMetadata>()
        .once()
        .returning(|_| Ok(None));

    let mut db = db.into_transactional();
    db.insert_messages(&9u64.into(), &messages[5..]).unwrap();

    let mut db = MockStorage::default();
    db.expect_insert::<Messages>().returning(|_, _| Ok(None));
    db.expect_commit().returning(|| Ok(()));
    db.expect_get::<RelayerMetadata>()
        .once()
        .returning(|_| Ok(Some(std::borrow::Cow::Owned(9u64.into()))));

    let mut db = db.into_transactional();
    db.insert_messages(&5u64.into(), &messages[..5]).unwrap();
}

#[test_case(None, 0, 0; "can set DA height to 0 when there is none available")]
#[test_case(None, 10, 10; "can set DA height to 10 when there is none available")]
#[test_case(0, 10, 10; "can set DA height to 10 when it is 0")]
#[test_case(0, None, 0; "inserts are bypassed when height goes from 0 to 0")]
#[test_case(10, 11, 11; "can set DA height to 11 when it is 10")]
#[test_case(11, None, 11; "inserts are bypassed when height goes from 11 to 11")]
#[test_case(11, None, 10; "inserts are bypassed when height reverted from 11 to 10")]
fn set_raises_da_height_monotonically(
    get: impl Into<Option<u64>>,
    inserts: impl Into<Option<u64>>,
    new_height: u64,
) {
    let mut db = MockStorage::default();
    if let Some(h) = inserts.into() {
        db.expect_insert::<RelayerMetadata>()
            .once()
            .withf(move |_, v| **v == h)
            .returning(|_, _| Ok(None));
    }
    let get = get.into().map(|g| Cow::Owned(g.into()));
    db.expect_get::<RelayerMetadata>()
        .once()
        .returning(move |_| Ok(get.clone()));
    db.expect_commit().returning(|| Ok(()));

    let mut db = db.into_transactional();
    db.set_finalized_da_height_to_at_least(&new_height.into())
        .unwrap();
}
