use quartzite_core::{AsObject, ObjectBase};
use quartzite_macros::Extend;

// AC1: #[root] struct generates As{Type} trait and self-ref impl.
#[derive(Extend)]
#[root]
struct Widget {
    #[base]
    object_base: ObjectBase,
    pub label: String,
}

#[test]
fn ac1_root_trait_and_self_impl() {
    let w = Widget {
        object_base: ObjectBase::new(),
        label: "btn".into(),
    };
    // AsWidget::widget() returns &Widget
    let r: &Widget = w.widget();
    assert_eq!(r.label, "btn");
}

// AC2: concrete type with #[base] generates delegation impl for parent trait.
#[derive(Extend)]
#[allow(dead_code)]
struct Button {
    #[base]
    widget: Widget,
    pub clicked: bool,
}

#[test]
fn ac2_delegation_impl() {
    let b = Button {
        widget: Widget {
            object_base: ObjectBase::new(),
            label: "ok".into(),
        },
        clicked: false,
    };
    // AsWidget is implemented for Button via delegation
    let w: &Widget = b.widget();
    assert_eq!(w.label, "ok");
    // AsObject is satisfied transitively through the blanket impl
    let _ob: &ObjectBase = b.object_base();
}

// AC5: #[mixin] generates only the leaf trait impl (not ancestor chain).
#[derive(Extend)]
#[root]
#[allow(dead_code)]
struct LayoutBase {
    pub width: u32,
    pub height: u32,
}

#[derive(Extend)]
struct Panel {
    #[base]
    widget: Widget,
    #[mixin]
    layout: LayoutBase,
}

#[test]
fn ac5_mixin_leaf_only() {
    let p = Panel {
        widget: Widget {
            object_base: ObjectBase::new(),
            label: "panel".into(),
        },
        layout: LayoutBase {
            width: 100,
            height: 200,
        },
    };
    // AsLayoutBase is implemented for Panel (leaf impl)
    let lb: &LayoutBase = p.layout_base();
    assert_eq!(lb.width, 100);
    // AsWidget is implemented for Panel via delegation (not mixin)
    let w: &Widget = p.widget();
    assert_eq!(w.label, "panel");
}
