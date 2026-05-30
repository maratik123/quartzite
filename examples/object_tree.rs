//! Example demonstrating `ObjectTree` insertion, named-child lookup, and traversal via the `quartzite` facade.

use quartzite::prelude::*;

#[derive(Extend, Object)]
#[root]
struct Node {
    /// Object infrastructure (name, dynamic dispatch) provided by `Extend`.
    #[base]
    object_base: ObjectBase,
}

#[object_impl]
impl Node {}

impl Node {
    #[inline]
    fn named(name: &str) -> Self {
        Self {
            object_base: ObjectBase::named(name),
        }
    }
}

fn main() {
    env_logger::init();
    let mut tree = ObjectTree::new();

    let root_id = tree.insert(Box::new(Node::named("root")), None);
    let child_id = tree.insert(Box::new(Node::named("child")), Some(root_id));
    let _grand_id = tree.insert(Box::new(Node::named("grandchild")), Some(child_id));

    println!("parent of child:   {:?}", tree.parent_of(child_id));
    println!("children of root:  {:?}", tree.children_of(root_id));
    println!("find 'grandchild': {:?}", tree.find_by_name("grandchild"));
    tree.with(root_id, |obj| {
        println!("root name:         {:?}", obj.object_base().name());
    });
}
