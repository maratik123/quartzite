//! Static reflection metadata types for the quartzite object system.
//!
//! These types are emitted by `#[derive(Object)]` / `#[object_impl]` and
//! stored as `'static` values. They can also be constructed manually for
//! types that do not use the derive macros.

/// Flags describing how a property can be accessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertyFlags {
    /// Property value can be read via `read_property`.
    pub readable: bool,
    /// Property value can be written via `write_property`.
    pub writable: bool,
    /// A change notification signal exists for this property.
    pub notify: bool,
    /// Property value is saved when the object is serialized.
    pub stored: bool,
    /// Property is visible in design tools.
    pub designable: bool,
    /// Property is intended for direct user interaction (e.g., a form field).
    pub user: bool,
    /// Property value never changes after construction; implies not writable.
    pub constant: bool,
}

impl PropertyFlags {
    /// All flags false — useful as a starting point before enabling specific flags.
    pub const fn none() -> Self {
        Self {
            readable: false,
            writable: false,
            notify: false,
            stored: false,
            designable: false,
            user: false,
            constant: false,
        }
    }

    /// The most common combination: readable + writable + stored + designable.
    pub const fn read_write() -> Self {
        Self {
            readable: true,
            writable: true,
            notify: false,
            stored: true,
            designable: true,
            user: false,
            constant: false,
        }
    }

    /// Readable, stored, designable, constant (not writable).
    pub const fn read_only() -> Self {
        Self {
            readable: true,
            writable: false,
            notify: false,
            stored: true,
            designable: true,
            user: false,
            constant: true,
        }
    }
}

impl Default for PropertyFlags {
    fn default() -> Self {
        Self::read_write()
    }
}

/// Static metadata for a single property.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertyMeta {
    /// The property name as it appears in the meta-system (e.g. `"count"`).
    pub name: &'static str,
    /// Rust type name of the property value (e.g. `"i64"`).
    pub type_name: &'static str,
    /// Access flags for this property.
    pub flags: PropertyFlags,
}

impl PropertyMeta {
    /// Construct a new `PropertyMeta` from its components.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::{PropertyFlags, PropertyMeta};
    ///
    /// let meta = PropertyMeta::new("count", "i64", PropertyFlags::read_write());
    /// assert_eq!(meta.name, "count");
    /// ```
    pub const fn new(name: &'static str, type_name: &'static str, flags: PropertyFlags) -> Self {
        Self {
            name,
            type_name,
            flags,
        }
    }
}

/// Static metadata for a single parameter (used in signals and methods).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParamMeta {
    /// Parameter name (e.g. `"value"`).
    pub name: &'static str,
    /// Rust type name of the parameter (e.g. `"i64"`).
    pub type_name: &'static str,
}

impl ParamMeta {
    /// Construct a new `ParamMeta`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::ParamMeta;
    ///
    /// let p = ParamMeta::new("value", "i64");
    /// assert_eq!(p.name, "value");
    /// assert_eq!(p.type_name, "i64");
    /// ```
    pub const fn new(name: &'static str, type_name: &'static str) -> Self {
        Self { name, type_name }
    }
}

/// Static metadata for a signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalMeta {
    /// Signal name as it appears in the meta-system (e.g. `"clicked"`).
    pub name: &'static str,
    /// Ordered list of parameter descriptors for this signal.
    pub params: &'static [ParamMeta],
}

impl SignalMeta {
    /// Construct a new `SignalMeta`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::SignalMeta;
    ///
    /// let s = SignalMeta::new("clicked", &[]);
    /// assert_eq!(s.name, "clicked");
    /// assert!(s.params.is_empty());
    /// ```
    pub const fn new(name: &'static str, params: &'static [ParamMeta]) -> Self {
        Self { name, params }
    }
}

/// Static metadata for a callable method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MethodMeta {
    /// Method name as it appears in the meta-system (e.g. `"click"`).
    pub name: &'static str,
    /// Ordered list of parameter descriptors for this method.
    pub params: &'static [ParamMeta],
    /// Rust type name of the return value (e.g. `"()"` or `"i64"`).
    pub return_type: &'static str,
}

impl MethodMeta {
    /// Construct a new `MethodMeta`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::MethodMeta;
    ///
    /// let m = MethodMeta::new("click", &[], "()");
    /// assert_eq!(m.name, "click");
    /// assert_eq!(m.return_type, "()");
    /// ```
    pub const fn new(
        name: &'static str,
        params: &'static [ParamMeta],
        return_type: &'static str,
    ) -> Self {
        Self {
            name,
            params,
            return_type,
        }
    }
}

/// A single enumerator entry: a name paired with an integer value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumEntry {
    /// Enumerator name as a string (e.g. `"Alpha"`).
    pub name: &'static str,
    /// Integer value of this enumerator.
    pub value: i64,
}

impl EnumEntry {
    /// Construct a new `EnumEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::EnumEntry;
    ///
    /// let e = EnumEntry::new("Alpha", 0);
    /// assert_eq!(e.name, "Alpha");
    /// assert_eq!(e.value, 0);
    /// ```
    pub const fn new(name: &'static str, value: i64) -> Self {
        Self { name, value }
    }
}

/// Static metadata for an enumeration type exposed via the meta-system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumMeta {
    /// Enum type name (e.g. `"State"`).
    pub name: &'static str,
    /// All enumerator entries for this enum.
    pub entries: &'static [EnumEntry],
}

impl EnumMeta {
    /// Construct a new `EnumMeta`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::{EnumEntry, EnumMeta};
    ///
    /// static ENTRIES: &[EnumEntry] = &[EnumEntry::new("On", 1), EnumEntry::new("Off", 0)];
    /// let em = EnumMeta::new("State", ENTRIES);
    /// assert_eq!(em.name, "State");
    /// assert_eq!(em.entries.len(), 2);
    /// ```
    pub const fn new(name: &'static str, entries: &'static [EnumEntry]) -> Self {
        Self { name, entries }
    }

    /// Find an entry by name; returns `None` if not present.
    pub fn entry_by_name(&self, name: &str) -> Option<EnumEntry> {
        self.entries.iter().find(|e| e.name == name).copied()
    }

    /// Find an entry by integer value; returns `None` if not present.
    pub fn entry_by_value(&self, value: i64) -> Option<EnumEntry> {
        self.entries.iter().find(|e| e.value == value).copied()
    }
}

/// The complete static reflection record for a type.
///
/// Each concrete object type provides exactly one `&'static MetaObject`.
/// All slices are `'static` so that the whole struct can be stored in a `static`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetaObject {
    /// The Rust type name of the described class (e.g. `"Button"`).
    pub class_name: &'static str,
    /// All properties exposed by this type.
    pub properties: &'static [PropertyMeta],
    /// All signals declared on this type.
    pub signals: &'static [SignalMeta],
    /// All invokable methods declared on this type.
    pub methods: &'static [MethodMeta],
    /// All enumerations declared on this type.
    pub enums: &'static [EnumMeta],
}

impl MetaObject {
    /// Construct a new `MetaObject` from its static components.
    ///
    /// Typically called once inside a `static` initializer generated by `#[derive(Object)]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::MetaObject;
    ///
    /// static META: MetaObject = MetaObject::new("MyType", &[], &[], &[], &[]);
    /// assert_eq!(META.class_name, "MyType");
    /// ```
    pub const fn new(
        class_name: &'static str,
        properties: &'static [PropertyMeta],
        signals: &'static [SignalMeta],
        methods: &'static [MethodMeta],
        enums: &'static [EnumMeta],
    ) -> Self {
        Self {
            class_name,
            properties,
            signals,
            methods,
            enums,
        }
    }

    /// Find property metadata by name.
    pub fn property(&self, name: &str) -> Option<PropertyMeta> {
        self.properties.iter().find(|p| p.name == name).copied()
    }

    /// Find signal metadata by name.
    pub fn signal(&self, name: &str) -> Option<SignalMeta> {
        self.signals.iter().find(|s| s.name == name).copied()
    }

    /// Find method metadata by name.
    pub fn method(&self, name: &str) -> Option<MethodMeta> {
        self.methods.iter().find(|m| m.name == name).copied()
    }

    /// Find enum metadata by name.
    pub fn enum_meta(&self, name: &str) -> Option<EnumMeta> {
        self.enums.iter().find(|e| e.name == name).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY_META: MetaObject = MetaObject::new("Empty", &[], &[], &[], &[]);

    #[test]
    fn empty_meta_object_constructs() {
        // Must not panic and must report zero entries.
        assert_eq!(EMPTY_META.class_name, "Empty");
        assert!(EMPTY_META.properties.is_empty());
        assert!(EMPTY_META.signals.is_empty());
        assert!(EMPTY_META.methods.is_empty());
        assert!(EMPTY_META.enums.is_empty());
    }

    #[test]
    fn property_meta_flags_readable_writable() {
        let flags = PropertyFlags::read_write();
        let prop = PropertyMeta::new("count", "i64", flags);
        assert!(prop.flags.readable);
        assert!(prop.flags.writable);
        assert!(!prop.flags.constant);
    }

    #[test]
    fn property_meta_flags_read_only_constant() {
        let flags = PropertyFlags::read_only();
        let prop = PropertyMeta::new("version", "i64", flags);
        assert!(prop.flags.readable);
        assert!(!prop.flags.writable);
        assert!(prop.flags.constant);
    }

    #[test]
    fn enum_meta_entry_lookup() {
        static ENTRIES: &[EnumEntry] = &[EnumEntry::new("Alpha", 0), EnumEntry::new("Beta", 1)];
        let em = EnumMeta::new("MyEnum", ENTRIES);

        let found = em.entry_by_name("Beta");
        assert!(found.is_some());
        assert_eq!(found.expect("Beta must exist").value, 1);

        assert!(em.entry_by_name("Gamma").is_none());
    }

    #[test]
    fn meta_object_property_lookup() {
        static PROPS: &[PropertyMeta] = &[PropertyMeta::new(
            "name",
            "String",
            PropertyFlags::read_write(),
        )];
        let meta = MetaObject::new("Widget", PROPS, &[], &[], &[]);

        assert!(meta.property("name").is_some());
        assert!(meta.property("missing").is_none());
        assert_eq!(meta.property("name").unwrap().type_name, "String");
    }

    #[test]
    fn meta_object_signal_lookup() {
        static SIGS: &[SignalMeta] = &[SignalMeta::new("clicked", &[])];
        let meta = MetaObject::new("Button", &[], SIGS, &[], &[]);

        assert!(meta.signal("clicked").is_some());
        assert!(meta.signal("missing").is_none());
        assert_eq!(meta.signal("clicked").unwrap().name, "clicked");
    }

    #[test]
    fn meta_object_method_lookup() {
        static METHODS: &[MethodMeta] = &[MethodMeta::new("click", &[], "()")];
        let meta = MetaObject::new("Button", &[], &[], METHODS, &[]);

        assert!(meta.method("click").is_some());
        assert!(meta.method("missing").is_none());
        assert_eq!(meta.method("click").unwrap().return_type, "()");
    }

    #[test]
    fn meta_object_enum_meta_lookup() {
        static ENTRIES: &[EnumEntry] = &[EnumEntry::new("On", 1), EnumEntry::new("Off", 0)];
        static ENUMS: &[EnumMeta] = &[EnumMeta::new("State", ENTRIES)];
        let meta = MetaObject::new("Device", &[], &[], &[], ENUMS);

        assert!(meta.enum_meta("State").is_some());
        assert!(meta.enum_meta("missing").is_none());
        assert_eq!(meta.enum_meta("State").unwrap().entries.len(), 2);
    }
}
