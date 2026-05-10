//! Static reflection metadata types for the quartzite object system.
//!
//! These types are emitted by `#[derive(Object)]` / `#[object_impl]` and
//! stored as `'static` values. They can also be constructed manually for
//! types that do not use the derive macros.

use enumflags2::{BitFlags, bitflags};

/// A single property-access capability. Combine with `|` to form [`PropertyFlags`].
///
/// # Examples
///
/// ```
/// use quartzite_core::meta::{PropertyFlag, PropertyFlags};
///
/// let f: PropertyFlags = PropertyFlag::Readable | PropertyFlag::Writable;
/// assert!(f.contains(PropertyFlag::Readable));
/// assert!(!f.contains(PropertyFlag::Notify));
/// ```
#[bitflags(default = Readable | Writable | Stored | Designable)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyFlag {
    /// Property value can be read via `read_property`.
    Readable = 0b0000_0001,
    /// Property value can be written via `write_property`.
    Writable = 0b0000_0010,
    /// A change notification signal exists for this property.
    Notify = 0b0000_0100,
    /// Property value is included when the object is snapshot-serialized.
    ///
    /// Properties with this flag are captured by
    /// `quartzite_runtime::snapshot::capture_object` and restored by
    /// `quartzite_runtime::snapshot::restore_object` / `restore_tree`.
    /// Requires the `serde` cargo feature on `quartzite-core`.
    Stored = 0b0000_1000,
    /// Property is visible in design tools.
    Designable = 0b0001_0000,
    /// Property is intended for direct user interaction (e.g., a form field).
    User = 0b0010_0000,
    /// Property value never changes after construction; implies not writable.
    Constant = 0b0100_0000,
}

/// A set of [`PropertyFlag`] values describing how a property can be accessed.
///
/// Construct with the named helpers on [`PropertyFlag`], or combine flags directly with `|`.
///
/// # Examples
///
/// ```
/// use quartzite_core::meta::{PropertyFlag, PropertyFlags};
///
/// let f = PropertyFlag::read_write();
/// assert!(f.contains(PropertyFlag::Readable));
/// assert!(f.contains(PropertyFlag::Writable));
/// assert!(!f.contains(PropertyFlag::Constant));
/// ```
pub type PropertyFlags = BitFlags<PropertyFlag>;

impl PropertyFlag {
    /// All flags unset — useful as a starting point before enabling specific flags.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::{PropertyFlag, PropertyFlags};
    ///
    /// let f = PropertyFlag::none();
    /// assert!(!f.contains(PropertyFlag::Readable));
    /// assert!(!f.contains(PropertyFlag::Writable));
    /// ```
    #[inline]
    pub const fn none() -> PropertyFlags {
        BitFlags::EMPTY
    }

    /// The most common combination: readable + writable + stored + designable.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::{PropertyFlag, PropertyFlags};
    ///
    /// let f = PropertyFlag::read_write();
    /// assert!(f.contains(PropertyFlag::Readable));
    /// assert!(f.contains(PropertyFlag::Writable));
    /// assert!(!f.contains(PropertyFlag::Constant));
    /// ```
    #[inline]
    pub const fn read_write() -> PropertyFlags {
        enumflags2::make_bitflags!(PropertyFlag::{Readable | Writable | Stored | Designable})
    }

    /// Readable, stored, designable, constant (not writable).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::{PropertyFlag, PropertyFlags};
    ///
    /// let f = PropertyFlag::read_only();
    /// assert!(f.contains(PropertyFlag::Readable));
    /// assert!(!f.contains(PropertyFlag::Writable));
    /// assert!(f.contains(PropertyFlag::Constant));
    /// ```
    #[inline]
    pub const fn read_only() -> PropertyFlags {
        enumflags2::make_bitflags!(PropertyFlag::{Readable | Stored | Designable | Constant})
    }
}

/// Static metadata for a single property.
///
/// # Examples
///
/// ```
/// use quartzite_core::meta::{PropertyFlag, PropertyMeta};
///
/// let meta = PropertyMeta::new("count", "i64", PropertyFlag::read_write());
/// assert_eq!(meta.name, "count");
/// ```
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
    /// Constructs a new `PropertyMeta` from its components.
    ///
    /// # Parameters
    ///
    /// - `name`: meta-system name of the property (e.g. `"count"`).
    /// - `type_name`: Rust type of the property value (e.g. `"i64"`).
    /// - `flags`: access flags controlling readability, writability, and storage.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::{PropertyFlag, PropertyMeta};
    ///
    /// let meta = PropertyMeta::new("count", "i64", PropertyFlag::read_write());
    /// assert_eq!(meta.name, "count");
    /// ```
    #[inline]
    pub const fn new(name: &'static str, type_name: &'static str, flags: PropertyFlags) -> Self {
        Self {
            name,
            type_name,
            flags,
        }
    }
}

/// Static metadata for a single parameter (used in signals and methods).
///
/// # Examples
///
/// ```
/// use quartzite_core::meta::ParamMeta;
///
/// let p = ParamMeta::new("value", "i64");
/// assert_eq!(p.name, "value");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParamMeta {
    /// Parameter name (e.g. `"value"`).
    pub name: &'static str,
    /// Rust type name of the parameter (e.g. `"i64"`).
    pub type_name: &'static str,
}

impl ParamMeta {
    /// Constructs a new `ParamMeta`.
    ///
    /// # Parameters
    ///
    /// - `name`: parameter name as exposed in the meta-system (e.g. `"value"`).
    /// - `type_name`: Rust type name of the parameter (e.g. `"i64"`).
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
    #[inline]
    pub const fn new(name: &'static str, type_name: &'static str) -> Self {
        Self { name, type_name }
    }
}

/// Static metadata for a signal.
///
/// # Examples
///
/// ```
/// use quartzite_core::meta::SignalMeta;
///
/// let s = SignalMeta::new("clicked", &[]);
/// assert_eq!(s.name, "clicked");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalMeta {
    /// Signal name as it appears in the meta-system (e.g. `"clicked"`).
    pub name: &'static str,
    /// Ordered list of parameter descriptors for this signal.
    pub params: &'static [ParamMeta],
}

impl SignalMeta {
    /// Constructs a new `SignalMeta`.
    ///
    /// # Parameters
    ///
    /// - `name`: meta-system signal name (e.g. `"clicked"`).
    /// - `params`: ordered slice of parameter descriptors; empty for nullary
    ///   signals.
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
    #[inline]
    pub const fn new(name: &'static str, params: &'static [ParamMeta]) -> Self {
        Self { name, params }
    }
}

/// Static metadata for a callable method.
///
/// # Examples
///
/// ```
/// use quartzite_core::meta::MethodMeta;
///
/// let m = MethodMeta::new("click", &[], "()");
/// assert_eq!(m.name, "click");
/// ```
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
    /// Constructs a new `MethodMeta`.
    ///
    /// # Parameters
    ///
    /// - `name`: meta-system method name (e.g. `"click"`).
    /// - `params`: ordered slice of parameter descriptors.
    /// - `return_type`: Rust type name of the return value (e.g. `"()"`,
    ///   `"i64"`).
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
    #[inline]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumEntry {
    /// Enumerator name as a string (e.g. `"Alpha"`).
    pub name: &'static str,
    /// Integer value of this enumerator.
    pub value: i64,
}

impl EnumEntry {
    /// Constructs a new `EnumEntry`.
    ///
    /// # Parameters
    ///
    /// - `name`: enumerator name as a `&'static str`.
    /// - `value`: integer discriminant of this enumerator.
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
    #[inline]
    pub const fn new(name: &'static str, value: i64) -> Self {
        Self { name, value }
    }
}

/// Returns `None` for any input; placeholder lookup used in hand-written
/// [`EnumMeta`] statics that have no name index.
///
/// # Parameters
///
/// - `_`: ignored entry-name query.
///
/// # Examples
///
/// ```
/// use quartzite_core::meta::noop_lookup_entry_by_name;
///
/// assert!(noop_lookup_entry_by_name("anything").is_none());
/// ```
#[inline]
pub fn noop_lookup_entry_by_name(_: &str) -> Option<EnumEntry> {
    None
}

/// Returns `None` for any input; placeholder lookup used in hand-written
/// [`EnumMeta`] statics that have no value index.
///
/// # Parameters
///
/// - `_`: ignored entry-value query.
///
/// # Examples
///
/// ```
/// use quartzite_core::meta::noop_lookup_entry_by_value;
///
/// assert!(noop_lookup_entry_by_value(42).is_none());
/// ```
#[inline]
pub fn noop_lookup_entry_by_value(_: i64) -> Option<EnumEntry> {
    None
}

/// Static metadata for an enumeration type exposed via the meta-system.
///
/// # Examples
///
/// ```
/// use quartzite_core::meta::{EnumEntry, EnumMeta, noop_lookup_entry_by_name, noop_lookup_entry_by_value};
///
/// static ENTRIES: &[EnumEntry] = &[EnumEntry::new("On", 1), EnumEntry::new("Off", 0)];
/// let em = EnumMeta::new("State", ENTRIES, noop_lookup_entry_by_name, noop_lookup_entry_by_value);
/// assert_eq!(em.name, "State");
/// ```
#[derive(Clone, Copy)]
pub struct EnumMeta {
    /// Enum type name (e.g. `"State"`).
    pub name: &'static str,
    /// All enumerator entries for this enum.
    pub entries: &'static [EnumEntry],
    /// Fast O(1) entry lookup by name; generated by `#[meta_enum]`.
    lookup_entry_by_name: fn(&str) -> Option<EnumEntry>,
    /// Fast O(1) entry lookup by integer value; generated by `#[meta_enum]`.
    lookup_entry_by_value: fn(i64) -> Option<EnumEntry>,
}

impl core::fmt::Debug for EnumMeta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EnumMeta")
            .field("name", &self.name)
            .field("entries", &self.entries)
            .finish_non_exhaustive()
    }
}

impl PartialEq for EnumMeta {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.entries == other.entries
    }
}

impl Eq for EnumMeta {}

impl EnumMeta {
    /// Constructs a new `EnumMeta`.
    ///
    /// The last two parameters are fast-path lookup functions generated by
    /// `#[meta_enum]`. Pass the `noop_lookup_entry_by_name` and
    /// `noop_lookup_entry_by_value` helpers for hand-written statics.
    ///
    /// # Parameters
    ///
    /// - `name`: meta-system enum type name (e.g. `"State"`).
    /// - `entries`: slice of all enumerator entries; ownership remains in
    ///   `'static` storage.
    /// - `lookup_entry_by_name`: fast-path name lookup; pass
    ///   [`noop_lookup_entry_by_name`] when no index is available.
    /// - `lookup_entry_by_value`: fast-path value lookup; pass
    ///   [`noop_lookup_entry_by_value`] when no index is available.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::{EnumEntry, EnumMeta, noop_lookup_entry_by_name, noop_lookup_entry_by_value};
    ///
    /// static ENTRIES: &[EnumEntry] = &[EnumEntry::new("On", 1), EnumEntry::new("Off", 0)];
    /// let em = EnumMeta::new("State", ENTRIES, noop_lookup_entry_by_name, noop_lookup_entry_by_value);
    /// assert_eq!(em.name, "State");
    /// assert_eq!(em.entries.len(), 2);
    /// ```
    #[inline]
    pub const fn new(
        name: &'static str,
        entries: &'static [EnumEntry],
        lookup_entry_by_name: fn(&str) -> Option<EnumEntry>,
        lookup_entry_by_value: fn(i64) -> Option<EnumEntry>,
    ) -> Self {
        Self {
            name,
            entries,
            lookup_entry_by_name,
            lookup_entry_by_value,
        }
    }

    /// Finds an entry by name; delegates to the fast-path lookup function.
    ///
    /// # Parameters
    ///
    /// - `name`: enumerator name to look up.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::{EnumEntry, EnumMeta, noop_lookup_entry_by_name, noop_lookup_entry_by_value};
    ///
    /// static ENTRIES: &[EnumEntry] = &[EnumEntry::new("Alpha", 0), EnumEntry::new("Beta", 1)];
    /// let em = EnumMeta::new("MyEnum", ENTRIES, noop_lookup_entry_by_name, noop_lookup_entry_by_value);
    /// assert!(em.entry_by_name("Beta").is_none()); // noop returns None
    /// ```
    #[inline]
    pub fn entry_by_name(&self, name: &str) -> Option<EnumEntry> {
        (self.lookup_entry_by_name)(name)
    }

    /// Finds an entry by integer value; delegates to the fast-path lookup function.
    ///
    /// # Parameters
    ///
    /// - `value`: integer discriminant to look up.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::{EnumEntry, EnumMeta, noop_lookup_entry_by_name, noop_lookup_entry_by_value};
    ///
    /// static ENTRIES: &[EnumEntry] = &[EnumEntry::new("Alpha", 0), EnumEntry::new("Beta", 1)];
    /// let em = EnumMeta::new("MyEnum", ENTRIES, noop_lookup_entry_by_name, noop_lookup_entry_by_value);
    /// assert!(em.entry_by_value(1).is_none()); // noop returns None
    /// ```
    #[inline]
    pub fn entry_by_value(&self, value: i64) -> Option<EnumEntry> {
        (self.lookup_entry_by_value)(value)
    }
}

/// Returns `None` for any input; placeholder property lookup used in hand-written
/// [`MetaObject`] statics that have no property index.
///
/// # Parameters
///
/// - `_`: ignored property-name query.
///
/// # Examples
///
/// ```
/// use quartzite_core::meta::noop_lookup_property;
///
/// assert!(noop_lookup_property("anything").is_none());
/// ```
#[inline]
pub fn noop_lookup_property(_: &str) -> Option<PropertyMeta> {
    None
}

/// Returns `None` for any input; placeholder signal lookup used in hand-written
/// [`MetaObject`] statics that have no signal index.
///
/// # Parameters
///
/// - `_`: ignored signal-name query.
///
/// # Examples
///
/// ```
/// use quartzite_core::meta::noop_lookup_signal;
///
/// assert!(noop_lookup_signal("anything").is_none());
/// ```
#[inline]
pub fn noop_lookup_signal(_: &str) -> Option<SignalMeta> {
    None
}

/// Returns `None` for any input; placeholder method lookup used in hand-written
/// [`MetaObject`] statics that have no method index.
///
/// # Parameters
///
/// - `_`: ignored method-name query.
///
/// # Examples
///
/// ```
/// use quartzite_core::meta::noop_lookup_method;
///
/// assert!(noop_lookup_method("anything").is_none());
/// ```
#[inline]
pub fn noop_lookup_method(_: &str) -> Option<MethodMeta> {
    None
}

/// Returns `None` for any input; placeholder enum lookup used in hand-written
/// [`MetaObject`] statics that have no enum index.
///
/// # Parameters
///
/// - `_`: ignored enum-name query.
///
/// # Examples
///
/// ```
/// use quartzite_core::meta::noop_lookup_enum;
///
/// assert!(noop_lookup_enum("anything").is_none());
/// ```
#[inline]
pub fn noop_lookup_enum(_: &str) -> Option<EnumMeta> {
    None
}

/// The complete static reflection record for a type.
///
/// Each concrete object type provides exactly one `&'static MetaObject`.
/// All slices are `'static` so that the whole struct can be stored in a `static`.
///
/// # Examples
///
/// ```
/// use quartzite_core::meta::{
///     MetaObject, noop_lookup_property, noop_lookup_signal,
///     noop_lookup_method, noop_lookup_enum,
/// };
///
/// static META: MetaObject = MetaObject::new(
///     "MyType", &[], &[], &[], &[],
///     noop_lookup_property, noop_lookup_signal, noop_lookup_method, noop_lookup_enum,
/// );
/// assert_eq!(META.class_name, "MyType");
/// ```
#[derive(Clone, Copy)]
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
    /// Fast O(1) property lookup by name; generated by `#[object_impl]`.
    lookup_property: fn(&str) -> Option<PropertyMeta>,
    /// Fast O(1) signal lookup by name; generated by `#[object_impl]`.
    lookup_signal: fn(&str) -> Option<SignalMeta>,
    /// Fast O(1) method lookup by name; generated by `#[object_impl]`.
    lookup_method: fn(&str) -> Option<MethodMeta>,
    /// Fast O(1) enum lookup by name; generated by `#[object_impl]`.
    lookup_enum: fn(&str) -> Option<EnumMeta>,
}

impl core::fmt::Debug for MetaObject {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MetaObject")
            .field("class_name", &self.class_name)
            .field("properties", &self.properties)
            .field("signals", &self.signals)
            .field("methods", &self.methods)
            .field("enums", &self.enums)
            .finish_non_exhaustive()
    }
}

impl PartialEq for MetaObject {
    fn eq(&self, other: &Self) -> bool {
        self.class_name == other.class_name
            && self.properties == other.properties
            && self.signals == other.signals
            && self.methods == other.methods
            && self.enums == other.enums
    }
}

impl Eq for MetaObject {}

impl MetaObject {
    /// Constructs a new `MetaObject` from its static components.
    ///
    /// The last four parameters are fast-path lookup functions generated by
    /// `#[object_impl]`. Pass the four `noop_lookup_*` helpers for hand-written statics.
    ///
    /// # Parameters
    ///
    /// - `class_name`: Rust type name of the described class (e.g. `"Button"`).
    /// - `properties`: slice of all property descriptors in declaration order.
    /// - `signals`: slice of all signal descriptors in declaration order.
    /// - `methods`: slice of all method descriptors in declaration order.
    /// - `enums`: slice of all nested enum descriptors in declaration order.
    /// - `lookup_property`: fast-path property lookup; pass
    ///   [`noop_lookup_property`] when no index is generated.
    /// - `lookup_signal`: fast-path signal lookup; pass [`noop_lookup_signal`]
    ///   when no index is generated.
    /// - `lookup_method`: fast-path method lookup; pass [`noop_lookup_method`]
    ///   when no index is generated.
    /// - `lookup_enum`: fast-path enum lookup; pass [`noop_lookup_enum`] when
    ///   no index is generated.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::{
    ///     MetaObject, noop_lookup_property, noop_lookup_signal,
    ///     noop_lookup_method, noop_lookup_enum,
    /// };
    ///
    /// static META: MetaObject = MetaObject::new(
    ///     "MyType", &[], &[], &[], &[],
    ///     noop_lookup_property, noop_lookup_signal, noop_lookup_method, noop_lookup_enum,
    /// );
    /// assert_eq!(META.class_name, "MyType");
    /// ```
    // Each slice + fn-pointer pair is a distinct, non-groupable concern; a builder would add
    // runtime overhead to a const fn used exclusively in static initialisers.
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub const fn new(
        class_name: &'static str,
        properties: &'static [PropertyMeta],
        signals: &'static [SignalMeta],
        methods: &'static [MethodMeta],
        enums: &'static [EnumMeta],
        lookup_property: fn(&str) -> Option<PropertyMeta>,
        lookup_signal: fn(&str) -> Option<SignalMeta>,
        lookup_method: fn(&str) -> Option<MethodMeta>,
        lookup_enum: fn(&str) -> Option<EnumMeta>,
    ) -> Self {
        Self {
            class_name,
            properties,
            signals,
            methods,
            enums,
            lookup_property,
            lookup_signal,
            lookup_method,
            lookup_enum,
        }
    }

    /// Finds property metadata by name.
    ///
    /// Delegates to the fast-path lookup function when available (generated by
    /// `#[object_impl]`); pass `noop_lookup_*` for types without generated lookups.
    ///
    /// # Parameters
    ///
    /// - `name`: property name to look up.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::{
    ///     MetaObject, PropertyFlag, PropertyMeta,
    ///     noop_lookup_signal, noop_lookup_method, noop_lookup_enum,
    /// };
    ///
    /// static PROP: PropertyMeta = PropertyMeta::new("count", "i64", PropertyFlag::read_write());
    /// static PROPS: &[PropertyMeta] = &[PROP];
    ///
    /// fn lookup_prop(name: &str) -> Option<PropertyMeta> {
    ///     match name { "count" => Some(PROP), _ => None }
    /// }
    ///
    /// let meta = MetaObject::new(
    ///     "Widget", PROPS, &[], &[], &[],
    ///     lookup_prop, noop_lookup_signal, noop_lookup_method, noop_lookup_enum,
    /// );
    /// assert!(meta.property("count").is_some());
    /// assert!(meta.property("missing").is_none());
    /// ```
    #[inline]
    pub fn property(&self, name: &str) -> Option<PropertyMeta> {
        (self.lookup_property)(name)
    }

    /// Finds signal metadata by name.
    ///
    /// Delegates to the fast-path lookup function when available (generated by
    /// `#[object_impl]`); pass `noop_lookup_*` for types without generated lookups.
    ///
    /// # Parameters
    ///
    /// - `name`: signal name to look up.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::{
    ///     MetaObject, SignalMeta,
    ///     noop_lookup_property, noop_lookup_method, noop_lookup_enum,
    /// };
    ///
    /// static SIG: SignalMeta = SignalMeta::new("clicked", &[]);
    /// static SIGS: &[SignalMeta] = &[SIG];
    ///
    /// fn lookup_sig(name: &str) -> Option<SignalMeta> {
    ///     match name { "clicked" => Some(SIG), _ => None }
    /// }
    ///
    /// let meta = MetaObject::new(
    ///     "Button", &[], SIGS, &[], &[],
    ///     noop_lookup_property, lookup_sig, noop_lookup_method, noop_lookup_enum,
    /// );
    /// assert!(meta.signal("clicked").is_some());
    /// assert!(meta.signal("missing").is_none());
    /// ```
    #[inline]
    pub fn signal(&self, name: &str) -> Option<SignalMeta> {
        (self.lookup_signal)(name)
    }

    /// Finds method metadata by name.
    ///
    /// Delegates to the fast-path lookup function when available (generated by
    /// `#[object_impl]`); pass `noop_lookup_*` for types without generated lookups.
    ///
    /// # Parameters
    ///
    /// - `name`: method name to look up.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::{
    ///     MetaObject, MethodMeta,
    ///     noop_lookup_property, noop_lookup_signal, noop_lookup_enum,
    /// };
    ///
    /// static METHOD: MethodMeta = MethodMeta::new("reset", &[], "()");
    /// static METHODS: &[MethodMeta] = &[METHOD];
    ///
    /// fn lookup_method(name: &str) -> Option<MethodMeta> {
    ///     match name { "reset" => Some(METHOD), _ => None }
    /// }
    ///
    /// let meta = MetaObject::new(
    ///     "Counter", &[], &[], METHODS, &[],
    ///     noop_lookup_property, noop_lookup_signal, lookup_method, noop_lookup_enum,
    /// );
    /// assert!(meta.method("reset").is_some());
    /// assert!(meta.method("missing").is_none());
    /// ```
    #[inline]
    pub fn method(&self, name: &str) -> Option<MethodMeta> {
        (self.lookup_method)(name)
    }

    /// Finds enum metadata by name.
    ///
    /// Delegates to the fast-path lookup function when available (generated by
    /// `#[object_impl]`); pass `noop_lookup_*` for types without generated lookups.
    ///
    /// # Parameters
    ///
    /// - `name`: enum type name to look up.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::meta::{
    ///     EnumEntry, EnumMeta, MetaObject,
    ///     noop_lookup_property, noop_lookup_signal, noop_lookup_method,
    ///     noop_lookup_entry_by_name, noop_lookup_entry_by_value,
    /// };
    ///
    /// static ENTRIES: &[EnumEntry] = &[EnumEntry::new("On", 1), EnumEntry::new("Off", 0)];
    /// static STATE_ENUM: EnumMeta =
    ///     EnumMeta::new("State", ENTRIES, noop_lookup_entry_by_name, noop_lookup_entry_by_value);
    /// static ENUMS: &[EnumMeta] = &[STATE_ENUM];
    ///
    /// fn lookup_enum(name: &str) -> Option<EnumMeta> {
    ///     match name { "State" => Some(STATE_ENUM), _ => None }
    /// }
    ///
    /// let meta = MetaObject::new(
    ///     "Device", &[], &[], &[], ENUMS,
    ///     noop_lookup_property, noop_lookup_signal, noop_lookup_method, lookup_enum,
    /// );
    /// assert!(meta.enum_meta("State").is_some());
    /// assert!(meta.enum_meta("missing").is_none());
    /// ```
    #[inline]
    pub fn enum_meta(&self, name: &str) -> Option<EnumMeta> {
        (self.lookup_enum)(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: build a MetaObject with all noop lookups.
    fn meta(
        class_name: &'static str,
        properties: &'static [PropertyMeta],
        signals: &'static [SignalMeta],
        methods: &'static [MethodMeta],
        enums: &'static [EnumMeta],
    ) -> MetaObject {
        MetaObject::new(
            class_name,
            properties,
            signals,
            methods,
            enums,
            noop_lookup_property,
            noop_lookup_signal,
            noop_lookup_method,
            noop_lookup_enum,
        )
    }

    // Helper: build an EnumMeta with noop lookups.
    fn enum_meta_noop(name: &'static str, entries: &'static [EnumEntry]) -> EnumMeta {
        EnumMeta::new(
            name,
            entries,
            noop_lookup_entry_by_name,
            noop_lookup_entry_by_value,
        )
    }

    const EMPTY_META: MetaObject = MetaObject::new(
        "Empty",
        &[],
        &[],
        &[],
        &[],
        noop_lookup_property,
        noop_lookup_signal,
        noop_lookup_method,
        noop_lookup_enum,
    );

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
        let flags = PropertyFlag::read_write();
        let prop = PropertyMeta::new("count", "i64", flags);
        assert!(prop.flags.contains(PropertyFlag::Readable));
        assert!(prop.flags.contains(PropertyFlag::Writable));
        assert!(!prop.flags.contains(PropertyFlag::Constant));
    }

    #[test]
    fn property_meta_flags_read_only_constant() {
        let flags = PropertyFlag::read_only();
        let prop = PropertyMeta::new("version", "i64", flags);
        assert!(prop.flags.contains(PropertyFlag::Readable));
        assert!(!prop.flags.contains(PropertyFlag::Writable));
        assert!(prop.flags.contains(PropertyFlag::Constant));
    }

    #[test]
    fn property_flag_none_is_empty() {
        assert!(PropertyFlag::none().is_empty());
    }

    #[test]
    fn property_flag_read_write_contains() {
        let f = PropertyFlag::read_write();
        assert!(f.contains(PropertyFlag::Readable));
        assert!(f.contains(PropertyFlag::Writable));
        assert!(f.contains(PropertyFlag::Stored));
        assert!(f.contains(PropertyFlag::Designable));
        assert!(!f.contains(PropertyFlag::Notify));
        assert!(!f.contains(PropertyFlag::User));
        assert!(!f.contains(PropertyFlag::Constant));
    }

    #[test]
    fn property_flag_read_only_contains() {
        let f = PropertyFlag::read_only();
        assert!(f.contains(PropertyFlag::Readable));
        assert!(!f.contains(PropertyFlag::Writable));
        assert!(f.contains(PropertyFlag::Stored));
        assert!(f.contains(PropertyFlag::Designable));
        assert!(f.contains(PropertyFlag::Constant));
        assert!(!f.contains(PropertyFlag::Notify));
        assert!(!f.contains(PropertyFlag::User));
    }

    #[test]
    fn property_flags_default_is_read_write() {
        assert_eq!(PropertyFlags::default(), PropertyFlag::read_write());
    }

    #[test]
    fn property_flag_const_constructors() {
        const NONE: PropertyFlags = PropertyFlag::none();
        const RW: PropertyFlags = PropertyFlag::read_write();
        const RO: PropertyFlags = PropertyFlag::read_only();
        assert!(NONE.is_empty());
        assert!(RW.contains(PropertyFlag::Writable));
        assert!(RO.contains(PropertyFlag::Constant));
    }

    #[test]
    fn enum_meta_entry_lookup_noop() {
        static ENTRIES: &[EnumEntry] = &[EnumEntry::new("Alpha", 0), EnumEntry::new("Beta", 1)];
        let em = enum_meta_noop("MyEnum", ENTRIES);
        // noop lookups always return None regardless of input
        assert!(em.entry_by_name("Beta").is_none());
        assert!(em.entry_by_name("Gamma").is_none());
        assert!(em.entry_by_value(1).is_none());
    }

    #[test]
    fn meta_object_property_lookup() {
        static PROPS: &[PropertyMeta] = &[PropertyMeta::new(
            "name",
            "String",
            PropertyFlag::read_write(),
        )];
        let m = meta("Widget", PROPS, &[], &[], &[]);
        // noop lookup returns None even for known properties
        assert!(m.property("name").is_none());
        assert!(m.property("missing").is_none());
    }

    #[test]
    fn meta_object_signal_lookup() {
        static SIGS: &[SignalMeta] = &[SignalMeta::new("clicked", &[])];
        let m = meta("Button", &[], SIGS, &[], &[]);
        assert!(m.signal("clicked").is_none());
        assert!(m.signal("missing").is_none());
    }

    #[test]
    fn meta_object_method_lookup() {
        static METHODS: &[MethodMeta] = &[MethodMeta::new("click", &[], "()")];
        let m = meta("Button", &[], &[], METHODS, &[]);
        assert!(m.method("click").is_none());
        assert!(m.method("missing").is_none());
    }

    #[test]
    fn meta_object_enum_meta_lookup() {
        static ENTRIES: &[EnumEntry] = &[EnumEntry::new("On", 1), EnumEntry::new("Off", 0)];
        static ENUMS: &[EnumMeta] = &[EnumMeta::new(
            "State",
            ENTRIES,
            noop_lookup_entry_by_name,
            noop_lookup_entry_by_value,
        )];
        let m = meta("Device", &[], &[], &[], ENUMS);
        assert!(m.enum_meta("State").is_none());
        assert!(m.enum_meta("missing").is_none());
    }

    #[test]
    fn meta_object_property_lookup_via_fn_pointer() {
        static PROP: PropertyMeta = PropertyMeta::new("count", "i64", PropertyFlag::read_write());
        static PROPS: &[PropertyMeta] = &[PROP];

        fn lookup_prop(name: &str) -> Option<PropertyMeta> {
            match name {
                "count" => Some(PROP),
                _ => None,
            }
        }

        let m = MetaObject::new(
            "Counter",
            PROPS,
            &[],
            &[],
            &[],
            lookup_prop,
            noop_lookup_signal,
            noop_lookup_method,
            noop_lookup_enum,
        );
        assert_eq!(m.property("count"), Some(PROP));
        assert!(m.property("missing").is_none());
    }

    #[test]
    fn enum_meta_entry_by_name_via_fn_pointer() {
        static ALPHA: EnumEntry = EnumEntry::new("Alpha", 0);
        static BETA: EnumEntry = EnumEntry::new("Beta", 1);
        static ENTRIES: &[EnumEntry] = &[ALPHA, BETA];

        fn lookup_by_name(name: &str) -> Option<EnumEntry> {
            match name {
                "Alpha" => Some(ALPHA),
                "Beta" => Some(BETA),
                _ => None,
            }
        }

        let em = EnumMeta::new(
            "MyEnum",
            ENTRIES,
            lookup_by_name,
            noop_lookup_entry_by_value,
        );
        assert_eq!(em.entry_by_name("Beta"), Some(BETA));
        assert!(em.entry_by_name("Gamma").is_none());
    }

    #[test]
    fn enum_meta_entry_by_value_via_fn_pointer() {
        static ALPHA: EnumEntry = EnumEntry::new("Alpha", 0);
        static BETA: EnumEntry = EnumEntry::new("Beta", 1);
        static ENTRIES: &[EnumEntry] = &[ALPHA, BETA];

        fn lookup_by_value(value: i64) -> Option<EnumEntry> {
            match value {
                0 => Some(ALPHA),
                1 => Some(BETA),
                _ => None,
            }
        }

        let em = EnumMeta::new(
            "MyEnum",
            ENTRIES,
            noop_lookup_entry_by_name,
            lookup_by_value,
        );
        assert_eq!(em.entry_by_value(1), Some(BETA));
        assert!(em.entry_by_value(99).is_none());
    }

    #[test]
    fn noop_lookups_always_return_none() {
        assert!(noop_lookup_property("anything").is_none());
        assert!(noop_lookup_signal("anything").is_none());
        assert!(noop_lookup_method("anything").is_none());
        assert!(noop_lookup_enum("anything").is_none());
        assert!(noop_lookup_entry_by_name("anything").is_none());
        assert!(noop_lookup_entry_by_value(42).is_none());
    }
}
