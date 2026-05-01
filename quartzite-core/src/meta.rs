/// Flags describing how a property can be accessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertyFlags {
    pub readable: bool,
    pub writable: bool,
    pub notify: bool,
    pub stored: bool,
    pub designable: bool,
    pub user: bool,
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
    pub name: &'static str,
    pub type_name: &'static str,
    pub flags: PropertyFlags,
}

impl PropertyMeta {
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
    pub name: &'static str,
    pub type_name: &'static str,
}

impl ParamMeta {
    pub const fn new(name: &'static str, type_name: &'static str) -> Self {
        Self { name, type_name }
    }
}

/// Static metadata for a signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalMeta {
    pub name: &'static str,
    pub params: &'static [ParamMeta],
}

impl SignalMeta {
    pub const fn new(name: &'static str, params: &'static [ParamMeta]) -> Self {
        Self { name, params }
    }
}

/// Static metadata for a callable method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MethodMeta {
    pub name: &'static str,
    pub params: &'static [ParamMeta],
    pub return_type: &'static str,
}

impl MethodMeta {
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
    pub name: &'static str,
    pub value: i64,
}

impl EnumEntry {
    pub const fn new(name: &'static str, value: i64) -> Self {
        Self { name, value }
    }
}

/// Static metadata for an enumeration type exposed via the meta-system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumMeta {
    pub name: &'static str,
    pub entries: &'static [EnumEntry],
}

impl EnumMeta {
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
    pub class_name: &'static str,
    pub properties: &'static [PropertyMeta],
    pub signals: &'static [SignalMeta],
    pub methods: &'static [MethodMeta],
    pub enums: &'static [EnumMeta],
}

impl MetaObject {
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
