use std::collections::HashMap;

use chrono::{Datelike, Timelike};
use gluesql::prelude::Value;
use godot::prelude::*;
use log::error;
use paste::paste;

use crate::db::Database;

use super::NewRunnerData;

type Uuid = GodotString;

/// Try to extract an expected value from a [gluesql::prelude::Value].
macro_rules! value {
    ($v:expr, $t:ident) => {{
        if let Value::$t(v) = $v {
            // TODO this is slightly unnecessary, somehow deref primitives and clone structs
            Some(v.clone())
        } else {
            let v = $v;
            log::error!("Unexpected value {v:?}, using default");

            None
        }
    }};
}

/// Helper macro for constructing structs out of SQL columns. Uses the `value!` macro internally for extracting
/// values and setting fields.
///
/// Godot types can be specified if the column data type is a [Value::Map]. The macro will automatically
/// try to construct the Godot type from the map.
macro_rules! from_iter {
    ($( [$col_pos:expr, $field:ident, $val_type:ident] ),+) => {
        fn from_iter<T: IntoIterator<Item = &'a Value>>(iter: T) -> Self {
            let mut data = Self::default();

            for (idx, v) in iter.into_iter().enumerate() {
                match idx {
                    $(
                        $col_pos => paste!(data.[<set_ $field>](from_iter!(@ v, $val_type))),
                    )+
                    _ => panic!("Too much data received {idx}"),
                }
            }

            data
        }
    };

    (@ $v:expr, I64) => {
        value!($v, I64).unwrap_or_default()
    };

    (@ $v:expr, F32) => {
        value!($v, F32).unwrap_or_default()
    };

    (@ $v:expr, Str) => {
        value!($v, Str).unwrap_or_default().into()
    };

    (@ $v:expr, Inet) => {
        value!($v, Inet).map(|v| v.to_string()).unwrap_or("127.0.0.1".into()).into()
    };

    (@ $v:expr, Bool) => {
        value!($v, Bool).unwrap_or_default()
    };

    (@ $v:expr, Timestamp) => {{
        let v = value!($v, Timestamp).unwrap_or_default();
        let mut d = Dictionary::new();
        d.insert("year", v.year());
        d.insert("month", v.month());
        d.insert("day", v.day());
        d.insert("hour", v.hour());
        d.insert("minute", v.minute());
        d.insert("second", v.second());

        d
    }};

    (@ $v:expr, Map) => {{
        let v = value!($v, Map).unwrap_or_default();
        let mut d = Dictionary::new();

        for (k, v) in v.iter() {
            d.insert(k.clone(), v.to_variant());
        }

        d
    }};

    (@ $v:expr, Vector2) => {{
        let v = value!($v, Map).unwrap_or_default();
        let mut vec2 = Vector2::default();

        if let Some(x) = v.get("x") {
            vec2.x = from_iter!(@ x, F32);
        }
        if let Some(y) = v.get("y") {
            vec2.y = from_iter!(@ y, F32);
        }

        vec2
    }};

    (@ $v:expr, Vector2i) => {{
        let v = value!($v, Map).unwrap_or_default();
        let mut vec2 = Vector2::default();

        if let Some(x) = v.get("x") {
            vec2.x = from_iter!(@ x, I32);
        }
        if let Some(y) = v.get("y") {
            vec2.y = from_iter!(@ y, I32);
        }

        vec2
    }};

    // TODO this is wrong, commenting out so i don't get confused
    // (@ $v:expr, Rect2) => {{
    //     let v = value!($v, Map).unwrap_or_default();
    //     let mut rect2 = Rect2::default();

    //     if let Some(x) = v.get("x") {
    //         vec2.x = from_iter!(@ x, F32);
    //     }
    //     if let Some(y) = v.get("y") {
    //         vec2.y = from_iter!(@ y, F32);
    //     }

    //     vec2
    // }};

    (@ $v:expr, Vector3) => {{
        let v = value!($v, Map).unwrap_or_default();
        let mut vec3 = Vector3::default();

        if let Some(x) = v.get("x") {
            vec3.x = from_iter!(@ x, F32);
        }
        if let Some(y) = v.get("y") {
            vec3.y = from_iter!(@ y, F32);
        }
        if let Some(z) = v.get("z") {
            vec3.z = from_iter!(@ z, F32);
        }

        vec3
    }};

    (@ $v:expr, Vector3i) => {{
        let v = value!($v, Map).unwrap_or_default();
        let mut vec3 = Vector3i::default();

        if let Some(x) = v.get("x") {
            vec3.x = from_iter!(@ x, I32);
        }
        if let Some(y) = v.get("y") {
            vec3.y = from_iter!(@ y, I32);
        }
        if let Some(z) = v.get("z") {
            vec3.z = from_iter!(@ z, I32);
        }

        vec3
    }};

    (@ $v:expr, Transform3D) => {{
        let v = value!($v, Map).unwrap_or_default();

        let mut a = Vector3::default();
        if let Some(x) = v.get("xx") {
            a.x = from_iter!(@ x, F32);
        }
        if let Some(y) = v.get("xy") {
            a.y = from_iter!(@ y, F32);
        }
        if let Some(z) = v.get("xz") {
            a.z = from_iter!(@ z, F32);
        }

        let mut b = Vector3::default();
        if let Some(x) = v.get("yx") {
            b.x = from_iter!(@ x, F32);
        }
        if let Some(y) = v.get("yy") {
            b.y = from_iter!(@ y, F32);
        }
        if let Some(z) = v.get("yz") {
            b.z = from_iter!(@ z, F32);
        }

        let mut c = Vector3::default();
        if let Some(x) = v.get("zx") {
            c.x = from_iter!(@ x, F32);
        }
        if let Some(y) = v.get("zy") {
            c.y = from_iter!(@ y, F32);
        }
        if let Some(z) = v.get("zz") {
            c.z = from_iter!(@ z, F32);
        }

        let mut o = Vector3::default();
        if let Some(x) = v.get("ox") {
            o.x = from_iter!(@ x, F32);
        }
        if let Some(y) = v.get("oy") {
            o.y = from_iter!(@ y, F32);
        }
        if let Some(z) = v.get("oz") {
            o.z = from_iter!(@ z, F32);
        }

        Transform3D::from_cols(a, b, c, o)
    }};
}

/// Database Access Object functions.
trait Dao
where
    Self: GodotClass,
{
    /// Pull all rows and return an [Array] of constructed Godot objects.
    fn pull_all(db: Gd<Database>) -> Array<Gd<Self>>;

    /// Try an pull a specific row and return a constructed Godot object or `null`.
    fn pull(db: Gd<Database>, id: Uuid) -> Option<Gd<Self>>;
}

/// Helper macro for binding [Dao] functions to Godot.
///
/// # Note
/// This _cannot_ be used if other functions need to be bound to Godot because of a godot-rust limitation on
/// multiple `[godot_api]` impl blocks.
macro_rules! bind_dao {
    ($struct:ident) => {
        #[godot_api]
        impl $struct {
            #[func(rename = pull_all)]
            fn pull_all_bound(db: Gd<Database>) -> Array<Gd<$struct>> {
                Self::pull_all(db)
            }

            #[func(rename = pull)]
            fn pull_bound(db: Gd<Database>, id: Uuid) -> Option<Gd<$struct>> {
                Self::pull(db, id)
            }
        }
    };
}

/// Local [ToVariant] trait so that it can be implemented on [Value].
///
/// # NOTE
/// _shakes fist at sky_
///
/// Curse you, orphan rules!
pub trait ToVariantDao {
    fn to_variant(&self) -> Variant;
}

impl ToVariantDao for Value {
    fn to_variant(&self) -> Variant {
        match self {
            Value::Bool(v) => Variant::from(*v),
            Value::I8(v) => Variant::from(*v),
            Value::I16(v) => Variant::from(*v),
            Value::I32(v) => Variant::from(*v),
            Value::I64(v) => Variant::from(*v),
            Value::I128(v) => Variant::from(*v as i64),
            Value::U8(v) => Variant::from(*v),
            Value::U16(v) => Variant::from(*v),
            Value::U32(v) => Variant::from(*v),
            Value::U64(v) => Variant::from(*v as u32),
            Value::U128(v) => Variant::from(*v as u32),
            Value::F32(v) => Variant::from(*v),
            Value::F64(v) => Variant::from(*v),
            Value::Decimal(v) => Variant::from(v.to_string()),
            Value::Str(v) => Variant::from(v.to_string()),
            Value::Bytea(v) => Variant::from(PackedByteArray::from_iter(v.clone())),
            Value::Inet(v) => Variant::from(v.to_string()),
            Value::Date(v) => {
                let mut d = Dictionary::new();
                d.insert("year", v.year());
                d.insert("month", v.month());
                d.insert("day", v.day());

                Variant::from(d)
            }
            Value::Timestamp(v) => {
                let mut d = Dictionary::new();
                d.insert("year", v.year());
                d.insert("month", v.month());
                d.insert("day", v.day());
                d.insert("hour", v.hour());
                d.insert("minute", v.minute());
                d.insert("second", v.second());

                Variant::from(d)
            }
            Value::Time(v) => {
                let mut d = Dictionary::new();
                d.insert("hour", v.hour());
                d.insert("minute", v.minute());
                d.insert("second", v.second());

                Variant::from(d)
            }
            Value::Interval(v) => Variant::from(format!("{v:?}")),
            Value::Uuid(v) => Variant::from(v.to_string()),
            Value::Map(v) => {
                let mut d = Dictionary::new();

                for (k, v) in v.iter() {
                    d.insert(k.clone(), v.to_variant());
                }

                Variant::from(d)
            }
            Value::List(v) => {
                let mut a = Array::new();

                for value in v.iter() {
                    a.push(value.to_variant());
                }

                Variant::from(a)
            }
            Value::Point(v) => Variant::from(Vector2::new(v.x as f32, v.y as f32)),
            Value::Null => Variant::nil(),
        }
    }
}

macro_rules! block_impl_trait {
    ( trait: $trait:ident, fn: $func:ident, ret: $ret:ty, $( [ $type_name:ty, $self:ident $block:block ] ),+ ) => {
        $(
            impl $trait for $type_name {
                fn $func(&self) -> $ret {
                    let $self = self;
                    $block
                }
            }
        )+
    };
}

/// Helper trait for converting values into GlueSql [Value]s.
trait ToGlueSqlValue {
    fn to_value(&self) -> Value;
}

impl ToGlueSqlValue for Variant {
    fn to_value(&self) -> Value {
        macro_rules! variant_to_value {
            ($type:ty) => {
                self.try_to::<$type>()
                    .unwrap_or_else(|e| {
                        error!("{e}");
                        <$type>::default()
                    })
                    .to_value()
            };
        }

        match self.get_type() {
            VariantType::Nil => Value::Null,
            VariantType::Bool => Value::Bool(self.to::<bool>()),
            VariantType::Int => Value::I64(self.to::<i64>()),
            VariantType::Float => Value::F32(self.to::<f32>()),
            VariantType::String => Value::Str(self.to_string()),
            VariantType::Vector2 => variant_to_value!(Vector2),
            VariantType::Vector2i => variant_to_value!(Vector2i),
            VariantType::Rect2 => variant_to_value!(Rect2),
            VariantType::Rect2i => variant_to_value!(Rect2i),
            VariantType::Vector3 => variant_to_value!(Vector3),
            VariantType::Vector3i => variant_to_value!(Vector3i),
            VariantType::Transform2D => variant_to_value!(Transform2D),
            VariantType::Vector4 => variant_to_value!(Vector4),
            VariantType::Vector4i => {
                panic!("This is broken due to Vector4i not being an EngineEnum as of Oct 16, 2023")
            }
            VariantType::Plane => variant_to_value!(Plane),
            VariantType::Quaternion => variant_to_value!(Quaternion),
            VariantType::Aabb => variant_to_value!(Aabb),
            VariantType::Basis => todo!(),
            VariantType::Transform3D => variant_to_value!(Transform3D),
            VariantType::Projection => todo!(),
            VariantType::Color => todo!(),
            VariantType::StringName => todo!(),
            VariantType::NodePath => todo!(),
            VariantType::Rid => todo!(),
            VariantType::Object => todo!(),
            VariantType::Callable => todo!(),
            VariantType::Signal => todo!(),
            VariantType::Dictionary => variant_to_value!(Dictionary),
            VariantType::Array => variant_to_value!(Array<Variant>),
            VariantType::PackedByteArray => todo!(),
            VariantType::PackedInt32Array => todo!(),
            VariantType::PackedInt64Array => todo!(),
            VariantType::PackedFloat32Array => todo!(),
            VariantType::PackedFloat64Array => todo!(),
            VariantType::PackedStringArray => todo!(),
            VariantType::PackedVector2Array => todo!(),
            VariantType::PackedVector3Array => todo!(),
            VariantType::PackedColorArray => todo!(),
        }
    }
}

block_impl_trait! {
    trait: ToGlueSqlValue,
    fn: to_value,
    ret: Value,
    [
        i32, this {
            Value::I32(*this)
        }
    ],
    [
        i64, this {
            Value::I64(*this)
        }
    ],
    [
        f32, this {
            Value::F32(*this)
        }
    ],
    [
        bool, this {
            Value::Bool(*this)
        }
    ],
    [
        GodotString, this {
            Value::Str(this.to_string())
        }
    ],
    [
        Vector2, this {
            this.to_value_map()
        }
    ],
    [
        Vector2i, this {
            this.to_value_map()
        }
    ],
    [
        Rect2, this {
            this.to_value_map()
        }
    ],
    [
        Rect2i, this {
            this.to_value_map()
        }
    ],
    [
        Vector3, this {
            this.to_value_map()
        }
    ],
    [
        Vector3i, this {
            this.to_value_map()
        }
    ],
    [
        Transform2D, this {
            this.to_value_map()
        }
    ],
    [
        Vector4, this {
            this.to_value_map()
        }
    ],
    [
        Vector4i, this {
            this.to_value_map()
        }
    ],
    [
        Plane, this {
            this.to_value_map()
        }
    ],
    [
        Quaternion, this {
            this.to_value_map()
        }
    ],
    [
        Aabb, this {
            this.to_value_map()
        }
    ],
    [
        Transform3D, this {
            this.to_value_map()
        }
    ],
    [
        Dictionary, this {
            this.to_value_map()
        }
    ],
    [
        Array<Variant>, this {
            let mut vec = Vec::new();

            for v in this.iter_shared() {
                vec.push(v.to_value());
            }

            Value::List(vec)
        }
    ]
}

trait ToGlueSqlMap {
    /// Convert the given type into a [HashMap];
    fn to_hash_map(&self) -> HashMap<String, Value>;

    /// Creates the GlueSql [Value::Map] variant. In general, this should not be modified.
    fn to_value_map(&self) -> Value {
        Value::Map(self.to_hash_map())
    }
}

block_impl_trait! {
    trait: ToGlueSqlMap,
    fn: to_hash_map,
    ret: HashMap<String, Value>,
    [
        Vector2, this {
            let mut map = HashMap::new();
            map.insert("x".into(), Value::F32(this.x));
            map.insert("y".into(), Value::F32(this.y));

            map
        }
    ],
    [
        Vector2i, this {
            let mut map = HashMap::new();
            map.insert("x".into(), Value::I32(this.x));
            map.insert("y".into(), Value::I32(this.y));

            map
        }
    ],
    [
        Rect2, this {
            let mut map = HashMap::new();
            map.insert("position".into(), this.position.to_value());
            map.insert("size".into(), this.size.to_value());

            map
        }
    ],
    [
        Rect2i, this {
            let mut map = HashMap::new();
            map.insert("position".into(), this.position.to_value());
            map.insert("size".into(), this.size.to_value());

            map
        }
    ],
    [
        Vector3, this {
            let mut map = HashMap::new();
            map.insert("x".into(), Value::F32(this.x));
            map.insert("y".into(), Value::F32(this.y));
            map.insert("z".into(), Value::F32(this.z));

            map
        }
    ],
    [
        Vector3i, this {
            let mut map = HashMap::new();
            map.insert("x".into(), Value::I32(this.x));
            map.insert("y".into(), Value::I32(this.y));
            map.insert("z".into(), Value::I32(this.z));

            map
        }
    ],
    [
        Transform2D, this {
            let mut map = HashMap::new();

            let a = this.a;
            map.insert("xx".into(), Value::F32(a.x));
            map.insert("xy".into(), Value::F32(a.y));

            let b = this.b;
            map.insert("yx".into(), Value::F32(b.x));
            map.insert("yy".into(), Value::F32(b.y));

            let o = this.origin;
            map.insert("ox".into(), Value::F32(o.x));
            map.insert("oy".into(), Value::F32(o.y));

            map
        }
    ],
    [
        Vector4, this {
            let mut map = HashMap::new();
            map.insert("x".into(), Value::F32(this.x));
            map.insert("y".into(), Value::F32(this.y));
            map.insert("z".into(), Value::F32(this.z));
            map.insert("w".into(), Value::F32(this.w));

            map
        }
    ],
    [
        Vector4i, this {
            let mut map = HashMap::new();
            map.insert("x".into(), Value::I32(this.x));
            map.insert("y".into(), Value::I32(this.y));
            map.insert("z".into(), Value::I32(this.z));
            map.insert("w".into(), Value::I32(this.w));

            map
        }
    ],
    [
        Plane, this {
            let mut map = HashMap::new();

            let normal = this.normal;
            map.insert("x".into(), Value::F32(normal.x));
            map.insert("y".into(), Value::F32(normal.y));
            map.insert("z".into(), Value::F32(normal.z));

            map.insert("d".into(), Value::F32(this.d));

            map
        }
    ],
    [
        Quaternion, this {
            let mut map = HashMap::new();
            map.insert("x".into(), Value::F32(this.x));
            map.insert("y".into(), Value::F32(this.y));
            map.insert("z".into(), Value::F32(this.z));
            map.insert("w".into(), Value::F32(this.w));

            map
        }
    ],
    [
        Aabb, this {
            let mut map = HashMap::new();

            map.insert("position".into(), this.position.to_value());
            map.insert("size".into(), this.size.to_value());

            map
        }
    ],
    [
        Transform3D, this {
            let mut map = HashMap::new();

            let a = this.basis.col_a();
            map.insert("xx".into(), Value::F32(a.x));
            map.insert("xy".into(), Value::F32(a.y));
            map.insert("xz".into(), Value::F32(a.z));

            let b = this.basis.col_b();
            map.insert("yx".into(), Value::F32(b.x));
            map.insert("yy".into(), Value::F32(b.y));
            map.insert("yz".into(), Value::F32(b.z));

            let c = this.basis.col_c();
            map.insert("zx".into(), Value::F32(c.x));
            map.insert("zy".into(), Value::F32(c.y));
            map.insert("zz".into(), Value::F32(c.z));

            let o = this.origin;
            map.insert("ox".into(), Value::F32(o.x));
            map.insert("oy".into(), Value::F32(o.y));
            map.insert("oz".into(), Value::F32(o.z));

            map
        }
    ],
    [
        Dictionary, this {
            let mut map = HashMap::new();

            for (k, v) in this.iter_shared() {
                map.insert(k.to_string(), v.to_value());
            }

            map
        }
    ]
}

#[derive(Debug, Default, GodotClass)]
#[property(name = name, type = GodotString, get = get_name, set = set_name)]
#[property(name = runner_path, type = GodotString, get = get_runner_path, set = set_runner_path)]
#[property(name = gui_path, type = GodotString, get = get_gui_path, set = set_gui_path)]
#[property(name = model_path, type = GodotString, get = get_model_path, set = set_model_path)]
pub struct RunnerData {
    data: NewRunnerData,
    #[var]
    preview_path: GodotString,
    #[var]
    is_favorite: bool,
    #[var]
    last_used: Dictionary,
}

impl From<NewRunnerData> for RunnerData {
    fn from(value: NewRunnerData) -> Self {
        Self {
            data: value,
            ..Default::default()
        }
    }
}

impl<'a> FromIterator<&'a Value> for RunnerData {
    from_iter![
        [0, name, Str],
        [1, runner_path, Str],
        [2, gui_path, Str],
        [3, model_path, Str],
        [4, preview_path, Str],
        [5, is_favorite, Bool],
        [6, last_used, Timestamp]
    ];
}

#[godot_api]
impl RefCountedVirtual for RunnerData {
    fn init(_base: godot::obj::Base<Self::Base>) -> Self {
        Self::default()
    }
}

impl Dao for RunnerData {
    fn pull_all(mut db: Gd<Database>) -> Array<Gd<Self>> {
        match db.bind_mut().select("select * from RunnerData") {
            Ok(v) => return Array::from_iter(v.iter().map(|v| Gd::new(RunnerData::from_iter(v)))),
            Err(e) => {}
        }

        todo!()
    }

    fn pull(mut db: Gd<Database>, id: Uuid) -> Option<Gd<Self>> {
        match db
            .bind_mut()
            .select(format!("select * from RunnerData where id = {id}"))
        {
            Ok(v) => {}
            Err(e) => {}
        }

        todo!()
    }
}

#[godot_api]
impl RunnerData {
    #[func(rename = pull_all)]
    fn pull_all_bound(db: Gd<Database>) -> Array<Gd<RunnerData>> {
        Self::pull_all(db)
    }

    #[func(rename = pull)]
    fn pull_bound(db: Gd<Database>, id: Uuid) -> Option<Gd<RunnerData>> {
        Self::pull(db, id)
    }

    #[func]
    fn get_name(&self) -> GodotString {
        self.data.name.clone()
    }

    #[func]
    fn set_name(&mut self, name: GodotString) {
        self.data.name = name;
    }

    #[func]
    fn get_runner_path(&self) -> GodotString {
        self.data.runner_path.clone()
    }

    #[func]
    fn set_runner_path(&mut self, runner_path: GodotString) {
        self.data.runner_path = runner_path;
    }

    #[func]
    fn get_gui_path(&self) -> GodotString {
        self.data.gui_path.clone()
    }

    #[func]
    fn set_gui_path(&mut self, gui_path: GodotString) {
        self.data.gui_path = gui_path;
    }

    #[func]
    fn get_model_path(&self) -> GodotString {
        self.data.model_path.clone()
    }

    #[func]
    fn set_model_path(&mut self, model_path: GodotString) {
        self.data.model_path = model_path;
    }
}

#[derive(Debug, Default, GodotClass)]
#[class(init)]
struct GeneralOptions {
    #[var]
    parent: Uuid,

    #[var]
    window_size: Vector2,
    #[var]
    window_screen: i64,
}

impl<'a> FromIterator<&'a Value> for GeneralOptions {
    from_iter![
        [0, parent, Str],
        [1, window_size, Vector2],
        [2, window_screen, I64]
    ];
}

impl Dao for GeneralOptions {
    fn pull_all(db: Gd<Database>) -> Array<Gd<Self>> {
        todo!()
    }

    fn pull(db: Gd<Database>, id: Uuid) -> Option<Gd<Self>> {
        todo!()
    }
}

bind_dao!(GeneralOptions);

#[derive(Debug, Default, GodotClass)]
#[class(init)]
struct IFacialMocapOptions {
    #[var]
    parent: Uuid,

    #[var]
    address: GodotString,
    #[var]
    port: i64,
}

impl<'a> FromIterator<&'a Value> for IFacialMocapOptions {
    from_iter![[0, parent, Str], [1, address, Inet], [2, port, I64]];
}

impl Dao for IFacialMocapOptions {
    fn pull_all(db: Gd<Database>) -> Array<Gd<Self>> {
        todo!()
    }

    fn pull(db: Gd<Database>, id: Uuid) -> Option<Gd<Self>> {
        todo!()
    }
}

bind_dao!(IFacialMocapOptions);

#[derive(Debug, Default, GodotClass)]
#[class(init)]
struct VTubeStudioOptions {
    #[var]
    parent: Uuid,

    #[var]
    address: GodotString,
    #[var]
    port: i64,
}

impl<'a> FromIterator<&'a Value> for VTubeStudioOptions {
    from_iter![[0, parent, Str], [1, address, Inet], [2, port, I64]];
}

impl Dao for VTubeStudioOptions {
    fn pull_all(db: Gd<Database>) -> Array<Gd<Self>> {
        todo!()
    }

    fn pull(db: Gd<Database>, id: Uuid) -> Option<Gd<Self>> {
        todo!()
    }
}

bind_dao!(VTubeStudioOptions);

#[derive(Debug, Default, GodotClass)]
#[class(init)]
struct MeowFaceOptions {
    #[var]
    parent: Uuid,

    #[var]
    address: GodotString,
    #[var]
    port: i64,
}

impl<'a> FromIterator<&'a Value> for MeowFaceOptions {
    from_iter![[0, parent, Str], [1, address, Inet], [2, port, I64]];
}

impl Dao for MeowFaceOptions {
    fn pull_all(db: Gd<Database>) -> Array<Gd<Self>> {
        todo!()
    }

    fn pull(db: Gd<Database>, id: Uuid) -> Option<Gd<Self>> {
        todo!()
    }
}

bind_dao!(MeowFaceOptions);

#[derive(Debug, Default, GodotClass)]
#[class(init)]
struct MediaPipeOptions {
    #[var]
    parent: Uuid,

    #[var]
    camera_resolution: Vector2,
}

impl<'a> FromIterator<&'a Value> for MediaPipeOptions {
    from_iter![[0, parent, Str], [1, camera_resolution, Vector2]];
}

impl Dao for MediaPipeOptions {
    fn pull_all(db: Gd<Database>) -> Array<Gd<Self>> {
        todo!()
    }

    fn pull(db: Gd<Database>, id: Uuid) -> Option<Gd<Self>> {
        todo!()
    }
}

bind_dao!(MediaPipeOptions);

#[derive(Debug, Default, GodotClass)]
#[class(init)]
struct Puppet3dOptions {
    #[var]
    parent: Uuid,

    #[var]
    head_bone: GodotString,
}

impl<'a> FromIterator<&'a Value> for Puppet3dOptions {
    from_iter![[0, parent, Str], [1, head_bone, Str]];
}

impl Dao for Puppet3dOptions {
    fn pull_all(db: Gd<Database>) -> Array<Gd<Self>> {
        todo!()
    }

    fn pull(db: Gd<Database>, id: Uuid) -> Option<Gd<Self>> {
        todo!()
    }
}

bind_dao!(Puppet3dOptions);

#[derive(Debug, Default, GodotClass)]
#[class(init)]
struct IkTargetTransformOptions {
    #[var]
    parent: Uuid,

    #[var]
    head: Transform3D,
    #[var]
    left_hand: Transform3D,
    #[var]
    right_hand: Transform3D,
    #[var]
    hips: Transform3D,
    #[var]
    left_foot: Transform3D,
    #[var]
    right_foot: Transform3D,
}

impl<'a> FromIterator<&'a Value> for IkTargetTransformOptions {
    from_iter![
        [0, head, Transform3D],
        [1, left_hand, Transform3D],
        [2, right_hand, Transform3D],
        [3, hips, Transform3D],
        [4, left_foot, Transform3D],
        [5, right_foot, Transform3D]
    ];
}

impl Dao for IkTargetTransformOptions {
    fn pull_all(db: Gd<Database>) -> Array<Gd<Self>> {
        todo!()
    }

    fn pull(db: Gd<Database>, id: Uuid) -> Option<Gd<Self>> {
        todo!()
    }
}

bind_dao!(IkTargetTransformOptions);

#[derive(Debug, Default, GodotClass)]
#[class(init)]
struct GlbPuppetOptions {
    #[var]
    parent: Uuid,
}

impl<'a> FromIterator<&'a Value> for GlbPuppetOptions {
    from_iter![[0, parent, Str]];
}

impl Dao for GlbPuppetOptions {
    fn pull_all(db: Gd<Database>) -> Array<Gd<Self>> {
        todo!()
    }

    fn pull(db: Gd<Database>, id: Uuid) -> Option<Gd<Self>> {
        todo!()
    }
}

bind_dao!(GlbPuppetOptions);
