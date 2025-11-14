use model_views::{Patch, Views};

#[derive(Debug, Views)]
#[cfg_attr(feature = "serde", views(serde = true))]
#[cfg_attr(not(feature = "serde"), views(serde = false))]
pub struct TestModel {
    #[views(get = "required", create = "forbidden", patch = "forbidden")]
    pub id: u64,
    #[views(get = "required")]
    pub name: String,
    #[views(get = "required", create = "optional", patch = "optional")]
    pub author: NestedModel,
}

#[derive(Debug, Views)]
#[cfg_attr(feature = "serde", views(serde = true))]
#[cfg_attr(not(feature = "serde"), views(serde = false))]
pub struct NestedModel {
    #[views(get = "required", create = "forbidden", patch = "forbidden")]
    pub id: u64,
    #[views(get = "required")]
    pub name: String,
}

#[test]
fn it_works() {
    let _ = TestModel {
        id: 1,
        name: "foo".to_string(),
        author: NestedModel {
            id: 1,
            name: "foo".to_string(),
        },
    };

    let _create = TestModelCreate {
        name: "foo".to_string(),
        author: None
    };

    let _patch = TestModelPatch {
        name: Patch::Update("foo".to_string()),
        author: Patch::Update(Some(
            NestedModelPatch {
                name: Patch::Update("foo".to_string()),
            }
        ))
    };

    let _read = TestModelGet {
        id: 1,
        name: "foo".to_string(),
        author: NestedModelGet {
            id: 1,
            name: "foo".to_string(),
        },
    };
}
