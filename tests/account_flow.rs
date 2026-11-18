use std::sync::{Arc, Barrier};

use mambofinance::{AccountKind, Application, CreateAccount};

#[test]
fn accounts_persist_with_stable_order_and_wire_safe_fields() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("finance.db");
    let application = Application::open(&path).expect("application");

    application
        .create_account(&CreateAccount {
            name: "Zulu Wallet".to_owned(),
            kind: AccountKind::Wallet,
            currency: "usd".to_owned(),
        })
        .expect("Zulu account");
    application
        .create_account(&CreateAccount {
            name: "Alpha Bank".to_owned(),
            kind: AccountKind::Bank,
            currency: "sgd".to_owned(),
        })
        .expect("Alpha account");
    drop(application);

    let accounts = Application::open(path)
        .expect("reopened application")
        .list_accounts()
        .expect("accounts")
        .accounts;

    assert_eq!(
        accounts
            .iter()
            .map(|account| account.name.as_str())
            .collect::<Vec<_>>(),
        ["Alpha Bank", "Zulu Wallet"]
    );
    for account in accounts {
        assert!(account.id.parse::<i64>().is_ok());
        assert_eq!(account.revision, 1);
        assert!(!account.archived);
        assert!(account.created_at.contains('T') && account.created_at.ends_with('Z'));
        assert_eq!(account.created_at, account.updated_at);
    }
}

#[test]
fn concurrent_normalized_duplicates_create_exactly_one_account() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let application = Application::open(directory.path().join("finance.db")).expect("application");
    let barrier = Arc::new(Barrier::new(3));

    let workers = ["Daily Bank", "  daily   bank  "].map(|name| {
        let application = application.clone();
        let barrier = Arc::clone(&barrier);
        std::thread::spawn(move || {
            barrier.wait();
            match application.create_account(&CreateAccount {
                name: name.to_owned(),
                kind: AccountKind::Bank,
                currency: "SGD".to_owned(),
            }) {
                Ok(_) => "created",
                Err(error) => error.code(),
            }
        })
    });
    barrier.wait();

    let mut outcomes = workers.map(|worker| worker.join().expect("account worker"));
    outcomes.sort_unstable();

    assert_eq!(outcomes, ["conflict", "created"]);
    assert_eq!(
        application
            .list_accounts()
            .expect("accounts")
            .accounts
            .len(),
        1
    );
}
