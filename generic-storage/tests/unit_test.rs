use generic_storage::{
    data_type::Person,
    serialize::{Borsh, Json, Wincode},
    storage::Storage,
};

#[test]
fn test_with_borsh() {
    let person = Person {
        name: "Luja".to_string(),
        age: 24,
    };
    let mut storage = Storage::new(Borsh);
    storage.save(&person).unwrap();
    let loaded = storage.load().unwrap();
    assert_eq!(loaded, person);
}
#[test]
fn test_with_serde() {
    let person = Person {
        name: "Luja".to_string(),
        age: 24,
    };
    let mut storage = Storage::new(Json);
    storage.save(&person).unwrap();
    let loaded = storage.load().unwrap();
    assert_eq!(loaded, person);
}
#[test]
fn test_with_wincode() {
    let person = Person {
        name: "Luja".to_string(),
        age: 24,
    };
    let mut storage = Storage::new(Wincode);
    storage.save(&person).unwrap();

    let borsh_storage = storage.convert(Borsh).unwrap();
    let loaded = borsh_storage.load().unwrap();
    assert_eq!(loaded, person);
}
