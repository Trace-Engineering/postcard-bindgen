//! Dart source generation.
//!
//! The generated library has no third-party dependencies and can therefore be
//! used by both standalone Dart applications and Flutter applications.

use crate::{
    registry::{BindingType, Container, ContainerCollection, EnumVariantType},
    type_info::{NumberMeta, ValueType},
};
use core::fmt::Write;

/// Settings for Dart binding generation.
#[derive(Debug, Clone)]
pub struct GenerationSettings {
    ser: bool,
    des: bool,
}

impl GenerationSettings {
    /// Enable serialization and deserialization.
    pub fn enable_all() -> Self {
        Self {
            ser: true,
            des: true,
        }
    }

    /// Enable or disable serialization.
    pub fn serialization(mut self, enabled: bool) -> Self {
        self.ser = enabled;
        self
    }

    /// Enable or disable deserialization.
    pub fn deserialization(mut self, enabled: bool) -> Self {
        self.des = enabled;
        self
    }
}

impl Default for GenerationSettings {
    fn default() -> Self {
        Self {
            ser: false,
            des: true,
        }
    }
}

/// Generate a single, dependency-free Dart library.
pub fn generate(containers: ContainerCollection, settings: &GenerationSettings) -> String {
    let containers = containers.all_containers().collect::<Vec<_>>();
    let mut out = String::from(
        "// GENERATED CODE - DO NOT MODIFY BY HAND.\n\
         import 'dart:convert';\n\
         import 'dart:typed_data';\n\n\
         class PostcardException implements Exception {\n\
         \x20 final String message;\n\
         \x20 const PostcardException(this.message);\n\
         \x20 @override String toString() => 'PostcardException: $message';\n\
         }\n\n\
         class PostcardRange<T> {\n\
         \x20 final T start;\n\
         \x20 final T end;\n\
         \x20 const PostcardRange(this.start, this.end);\n\
         }\n\n",
    );

    for container in &containers {
        gen_model(&mut out, container);
    }
    if settings.ser {
        out.push_str(SERIALIZER);
        for container in &containers {
            gen_ser_fn(&mut out, container);
        }
        gen_serialize_dispatch(&mut out, &containers);
    }
    if settings.des {
        out.push_str(DESERIALIZER);
        for container in &containers {
            gen_des_fn(&mut out, container);
        }
        gen_deserialize_dispatch(&mut out, &containers);
    }
    out
}

fn dart_name(path: &crate::path::Path<'_, '_>, name: &str) -> String {
    path.parts()
        .filter(|part| !part.is_empty())
        .skip(1)
        .chain(core::iter::once(name))
        .map(ident)
        .collect::<Vec<_>>()
        .join("_")
}

fn container_name(container: &Container) -> String {
    dart_name(&container.path, container.name)
}

fn ident(value: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "abstract",
        "as",
        "assert",
        "async",
        "await",
        "base",
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "covariant",
        "default",
        "deferred",
        "do",
        "dynamic",
        "else",
        "enum",
        "export",
        "extends",
        "extension",
        "external",
        "factory",
        "false",
        "final",
        "finally",
        "for",
        "Function",
        "get",
        "hide",
        "if",
        "implements",
        "import",
        "in",
        "interface",
        "is",
        "late",
        "library",
        "mixin",
        "new",
        "null",
        "of",
        "on",
        "operator",
        "part",
        "required",
        "rethrow",
        "return",
        "sealed",
        "set",
        "show",
        "static",
        "super",
        "switch",
        "sync",
        "this",
        "throw",
        "true",
        "try",
        "typedef",
        "var",
        "void",
        "when",
        "while",
        "with",
        "yield",
    ];
    if KEYWORDS.contains(&value) {
        format!("{value}_")
    } else {
        value.to_owned()
    }
}

fn dart_type(ty: &ValueType) -> String {
    match ty {
        ValueType::Number(NumberMeta::FloatingPoint { .. }) => "double".into(),
        ValueType::Number(NumberMeta::Integer { bytes, .. }) if *bytes > 4 => "BigInt".into(),
        ValueType::Number(_) => "int".into(),
        ValueType::Bool(_) => "bool".into(),
        ValueType::String(_) => "String".into(),
        ValueType::Object(v) => dart_name(&v.path, v.name),
        ValueType::Optional(v) => format!("{}?", dart_type(&v.inner)),
        ValueType::Array(v) => format!("List<{}>", dart_type(&v.items_type)),
        ValueType::Map(v) => format!(
            "Map<{}, {}>",
            dart_type(&v.key_type),
            dart_type(&v.value_type)
        ),
        ValueType::Range(v) => format!("PostcardRange<{}>", dart_type(&v.bounds_type)),
        ValueType::Tuple(v) => {
            let fields = v.items_types.iter().map(dart_type).collect::<Vec<_>>();
            if fields.len() == 1 {
                format!("({},)", fields[0])
            } else {
                format!("({})", fields.join(", "))
            }
        }
    }
}

fn gen_model(out: &mut String, container: &Container) {
    let name = container_name(container);
    match &container.r#type {
        BindingType::Struct(v) => {
            writeln!(out, "class {name} {{").unwrap();
            for f in &v.fields {
                writeln!(out, "  final {} {};", dart_type(&f.v_type), ident(f.name)).unwrap();
            }
            write!(out, "  const {name}({{").unwrap();
            for f in &v.fields {
                write!(out, "required this.{}, ", ident(f.name)).unwrap();
            }
            out.push_str("});\n}\n\n");
        }
        BindingType::TupleStruct(v) => {
            writeln!(out, "class {name} {{").unwrap();
            for (i, f) in v.fields.iter().enumerate() {
                writeln!(out, "  final {} item{};", dart_type(f), i).unwrap();
            }
            write!(out, "  const {name}(").unwrap();
            for i in 0..v.fields.len() {
                write!(out, "this.item{}, ", i).unwrap();
            }
            out.push_str(");\n}\n\n");
        }
        BindingType::UnitStruct(_) => {
            writeln!(out, "class {name} {{ const {name}(); }}\n").unwrap();
        }
        BindingType::Enum(v) => {
            writeln!(out, "sealed class {name} {{ const {name}(); }}").unwrap();
            for variant in &v.variants {
                let variant_name = format!("{name}_{}", variant.name);
                writeln!(out, "class {variant_name} extends {name} {{").unwrap();
                match &variant.inner_type {
                    EnumVariantType::Empty => writeln!(out, "  const {variant_name}();").unwrap(),
                    EnumVariantType::Tuple(fields) => {
                        for (i, f) in fields.iter().enumerate() {
                            writeln!(out, "  final {} item{};", dart_type(f), i).unwrap();
                        }
                        write!(out, "  const {variant_name}(").unwrap();
                        for i in 0..fields.len() {
                            write!(out, "this.item{}, ", i).unwrap();
                        }
                        out.push_str(");\n");
                    }
                    EnumVariantType::NewType(fields) => {
                        for f in fields {
                            writeln!(out, "  final {} {};", dart_type(&f.v_type), ident(f.name))
                                .unwrap();
                        }
                        write!(out, "  const {variant_name}({{").unwrap();
                        for f in fields {
                            write!(out, "required this.{}, ", ident(f.name)).unwrap();
                        }
                        out.push_str("});\n");
                    }
                }
                out.push_str("}\n");
            }
            out.push('\n');
        }
    }
}

fn ser_expr(ty: &ValueType, value: &str) -> String {
    match ty {
        ValueType::Number(NumberMeta::Integer {
            bytes,
            signed,
            zero_able,
        }) => {
            format!("s.integer({bytes}, {signed}, {zero_able}, {value});")
        }
        ValueType::Number(NumberMeta::FloatingPoint { bytes }) => {
            format!("s.float{bits}({value});", bits = bytes * 8)
        }
        ValueType::Bool(_) => format!("s.boolean({value});"),
        ValueType::String(v) => format!(
            "s.string({value}, {});",
            v.max_length
                .map(|n| n.to_string())
                .unwrap_or_else(|| "null".into())
        ),
        ValueType::Object(v) => {
            format!("_serialize{}(s, {value});", dart_name(&v.path, v.name))
        }
        ValueType::Optional(v) => format!(
            "if ({value} == null) {{ s.byte(0); }} else {{ s.byte(1); {} }}",
            ser_expr(&v.inner, &format!("{value}!"))
        ),
        ValueType::Array(v) => {
            let len = if let Some(length) = v.length {
                format!(
                    "if ({value}.length != {}) {{ throw const PostcardException('Invalid fixed array length'); }} ",
                    length
                )
            } else {
                format!(
                    "s.length({value}.length, {}); ",
                    v.max_length
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "null".into())
                )
            };
            format!(
                "{{ {len}for (final e in {value}) {{ {} }} }}",
                ser_expr(&v.items_type, "e")
            )
        }
        ValueType::Map(v) => format!(
            "{{ s.length({value}.length, {}); for (final e in {value}.entries) {{ {} {} }} }}",
            v.max_length
                .map(|n| n.to_string())
                .unwrap_or_else(|| "null".into()),
            ser_expr(&v.key_type, "e.key"),
            ser_expr(&v.value_type, "e.value")
        ),
        ValueType::Range(v) => format!(
            "{} {}",
            ser_expr(&v.bounds_type, &format!("{value}.start")),
            ser_expr(&v.bounds_type, &format!("{value}.end"))
        ),
        ValueType::Tuple(v) => v
            .items_types
            .iter()
            .enumerate()
            .map(|(i, t)| ser_expr(t, &format!("{value}.${}", i + 1)))
            .collect::<Vec<_>>()
            .join(" "),
    }
}

fn des_expr(ty: &ValueType) -> String {
    match ty {
        ValueType::Number(NumberMeta::Integer {
            bytes,
            signed,
            zero_able,
        }) => format!(
            "d.integer({bytes}, {signed}, {zero_able}){}",
            if *bytes > 4 { "" } else { ".toInt()" }
        ),
        ValueType::Number(NumberMeta::FloatingPoint { bytes }) => format!("d.float{}()", bytes * 8),
        ValueType::Bool(_) => "d.boolean()".into(),
        ValueType::String(v) => format!(
            "d.string({})",
            v.max_length
                .map(|n| n.to_string())
                .unwrap_or_else(|| "null".into())
        ),
        ValueType::Object(v) => format!("_deserialize{}(d)", dart_name(&v.path, v.name)),
        ValueType::Optional(v) => format!("d.optional(() => {})", des_expr(&v.inner)),
        ValueType::Array(v) => {
            let len = v.length.map(|n| n.to_string()).unwrap_or_else(|| {
                format!(
                    "d.length({})",
                    v.max_length
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "null".into())
                )
            });
            format!(
                "List.generate({len}, (_) => {}, growable: false)",
                des_expr(&v.items_type)
            )
        }
        ValueType::Map(v) => format!(
            "{{ for (var i = 0, n = d.length({}); i < n; i++) {}: {} }}",
            v.max_length
                .map(|n| n.to_string())
                .unwrap_or_else(|| "null".into()),
            des_expr(&v.key_type),
            des_expr(&v.value_type)
        ),
        ValueType::Range(v) => format!(
            "PostcardRange({}, {})",
            des_expr(&v.bounds_type),
            des_expr(&v.bounds_type)
        ),
        ValueType::Tuple(v) => {
            let fields = v.items_types.iter().map(des_expr).collect::<Vec<_>>();
            if fields.len() == 1 {
                format!("({},)", fields[0])
            } else {
                format!("({})", fields.join(", "))
            }
        }
    }
}

fn gen_ser_fn(out: &mut String, container: &Container) {
    let name = container_name(container);
    writeln!(
        out,
        "void _serialize{name}(_PostcardSerializer s, {name} v) {{"
    )
    .unwrap();
    match &container.r#type {
        BindingType::Struct(v) => {
            for f in &v.fields {
                writeln!(
                    out,
                    "  {}",
                    ser_expr(&f.v_type, &format!("v.{}", ident(f.name)))
                )
                .unwrap();
            }
        }
        BindingType::TupleStruct(v) => {
            for (i, f) in v.fields.iter().enumerate() {
                writeln!(out, "  {}", ser_expr(f, &format!("v.item{i}"))).unwrap();
            }
        }
        BindingType::UnitStruct(_) => {}
        BindingType::Enum(v) => {
            for variant in &v.variants {
                let vn = format!("{name}_{}", variant.name);
                writeln!(
                    out,
                    "  {}if (v is {vn}) {{",
                    if variant.index == 0 { "" } else { "else " }
                )
                .unwrap();
                writeln!(out, "    s.length({});", variant.index).unwrap();
                match &variant.inner_type {
                    EnumVariantType::Empty => {}
                    EnumVariantType::Tuple(fields) => {
                        for (i, f) in fields.iter().enumerate() {
                            writeln!(out, "    {}", ser_expr(f, &format!("v.item{i}"))).unwrap();
                        }
                    }
                    EnumVariantType::NewType(fields) => {
                        for f in fields {
                            writeln!(
                                out,
                                "    {}",
                                ser_expr(&f.v_type, &format!("v.{}", ident(f.name)))
                            )
                            .unwrap();
                        }
                    }
                }
                out.push_str("  }\n");
            }
            out.push_str("  else { throw PostcardException('Unknown enum variant'); }\n");
        }
    }
    out.push_str("}\n\n");
}

fn gen_des_fn(out: &mut String, container: &Container) {
    let name = container_name(container);
    writeln!(out, "{name} _deserialize{name}(_PostcardDeserializer d) {{").unwrap();
    match &container.r#type {
        BindingType::Struct(v) => {
            writeln!(out, "  return {name}(").unwrap();
            for f in &v.fields {
                writeln!(out, "    {}: {},", ident(f.name), des_expr(&f.v_type)).unwrap();
            }
            out.push_str("  );\n");
        }
        BindingType::TupleStruct(v) => {
            write!(out, "  return {name}(").unwrap();
            for f in &v.fields {
                write!(out, "{}, ", des_expr(f)).unwrap();
            }
            out.push_str(");\n");
        }
        BindingType::UnitStruct(_) => writeln!(out, "  return const {name}();").unwrap(),
        BindingType::Enum(v) => {
            out.push_str("  switch (d.length()) {\n");
            for variant in &v.variants {
                let vn = format!("{name}_{}", variant.name);
                write!(out, "    case {}: return {vn}(", variant.index).unwrap();
                match &variant.inner_type {
                    EnumVariantType::Empty => {}
                    EnumVariantType::Tuple(fields) => {
                        for f in fields {
                            write!(out, "{}, ", des_expr(f)).unwrap();
                        }
                    }
                    EnumVariantType::NewType(fields) => {
                        for f in fields {
                            write!(out, "{}: {}, ", ident(f.name), des_expr(&f.v_type)).unwrap();
                        }
                    }
                }
                out.push_str(");\n");
            }
            out.push_str("    default: throw PostcardException('Unknown enum variant');\n  }\n");
        }
    }
    out.push_str("}\n\n");
}

fn gen_serialize_dispatch(out: &mut String, containers: &[Container]) {
    out.push_str("Uint8List serialize(Object value) {\n  final s = _PostcardSerializer();\n");
    if containers.is_empty() {
        out.push_str(
            "  throw const PostcardException('No serializable types were generated');\n}\n\n",
        );
        return;
    }
    for (i, container) in containers.iter().enumerate() {
        let name = container_name(container);
        writeln!(
            out,
            "  {}if (value is {name}) {{ _serialize{name}(s, value); }}",
            if i == 0 { "" } else { "else " },
        )
        .unwrap();
    }
    out.push_str("  else { throw PostcardException('Type is not serializable'); }\n  return s.finish();\n}\n\n");
}

fn gen_deserialize_dispatch(out: &mut String, containers: &[Container]) {
    out.push_str("T deserialize<T>(Uint8List bytes) {\n  final d = _PostcardDeserializer(bytes);\n  late final Object value;\n");
    if containers.is_empty() {
        out.push_str(
            "  throw const PostcardException('No deserializable types were generated');\n}\n",
        );
        return;
    }
    for (i, container) in containers.iter().enumerate() {
        let name = container_name(container);
        writeln!(
            out,
            "  {}if (T == {name}) {{ value = _deserialize{name}(d); }}",
            if i == 0 { "" } else { "else " },
        )
        .unwrap();
    }
    out.push_str("  else { throw PostcardException('Type is not deserializable'); }\n  d.finish();\n  return value as T;\n}\n");
}

const SERIALIZER: &str = r#"
class _PostcardSerializer {
  final BytesBuilder _out = BytesBuilder(copy: false);
  void byte(int value) => _out.addByte(value & 0xff);
  void length(int value, [int? maxLength]) {
    if (maxLength != null && value > maxLength) {
      throw const PostcardException('Collection exceeds its maximum length');
    }
    integer(4, false, true, value);
  }
  void integer(int bytes, bool signed, bool zeroAble, Object value) {
    var n = value is BigInt ? value : BigInt.from(value as int);
    final bits = bytes * 8;
    final min = signed ? -(BigInt.one << (bits - 1)) : BigInt.zero;
    final max = signed ? (BigInt.one << (bits - 1)) - BigInt.one : (BigInt.one << bits) - BigInt.one;
    if (n < min || n > max) throw const PostcardException('Integer out of range');
    if (!zeroAble && n == BigInt.zero) throw const PostcardException('Expected a non-zero integer');
    if (bytes == 1) { byte(n.toUnsigned(8).toInt()); return; }
    if (signed) n = (n << 1) ^ (n >> (bits - 1));
    do {
      var b = (n & BigInt.from(0x7f)).toInt();
      n >>= 7;
      if (n != BigInt.zero) b |= 0x80;
      byte(b);
    } while (n != BigInt.zero);
  }
  void float32(double value) {
    final b = ByteData(4)..setFloat32(0, value, Endian.little);
    _out.add(b.buffer.asUint8List());
  }
  void float64(double value) {
    final b = ByteData(8)..setFloat64(0, value, Endian.little);
    _out.add(b.buffer.asUint8List());
  }
  void boolean(bool value) => byte(value ? 1 : 0);
  void string(String value, int? maxLength) {
    final encoded = utf8.encode(value);
    length(encoded.length, maxLength);
    _out.add(encoded);
  }
  Uint8List finish() => _out.takeBytes();
}

"#;

const DESERIALIZER: &str = r#"
class _PostcardDeserializer {
  final Uint8List _bytes;
  int _offset = 0;
  _PostcardDeserializer(this._bytes);
  int byte() {
    if (_offset >= _bytes.length) throw const PostcardException('Input buffer too small');
    return _bytes[_offset++];
  }
  int length([int? maxLength]) {
    final value = integer(4, false, true).toInt();
    if (maxLength != null && value > maxLength) {
      throw const PostcardException('Collection exceeds its maximum length');
    }
    return value;
  }
  BigInt integer(int bytes, bool signed, bool zeroAble) {
    if (bytes == 1) {
      final n = BigInt.from(byte());
      final value = signed && n >= BigInt.from(128) ? n - BigInt.from(256) : n;
      if (!zeroAble && value == BigInt.zero) throw const PostcardException('Expected a non-zero integer');
      return value;
    }
    var out = BigInt.zero;
    final max = (bytes * 8 + 6) ~/ 7;
    for (var i = 0; i < max; i++) {
      final b = byte();
      if (i == max - 1 && (b & 0x7f) >= (1 << ((bytes * 8) % 7))) {
        throw const PostcardException('Invalid varint');
      }
      out |= BigInt.from(b & 0x7f) << (7 * i);
      if ((b & 0x80) == 0) {
        final value = signed ? (out >> 1) ^ -(out & BigInt.one) : out;
        if (!zeroAble && value == BigInt.zero) throw const PostcardException('Expected a non-zero integer');
        return value;
      }
    }
    throw const PostcardException('Invalid varint');
  }
  double float32() {
    _need(4); final v = ByteData.sublistView(_bytes, _offset, _offset + 4).getFloat32(0, Endian.little); _offset += 4; return v;
  }
  double float64() {
    _need(8); final v = ByteData.sublistView(_bytes, _offset, _offset + 8).getFloat64(0, Endian.little); _offset += 8; return v;
  }
  bool boolean() {
    final v = byte();
    if (v > 1) throw const PostcardException('Invalid bool');
    return v == 1;
  }
  T? optional<T>(T Function() read) {
    final tag = byte();
    if (tag == 0) return null;
    if (tag == 1) return read();
    throw const PostcardException('Invalid option');
  }
  String string(int? maxLength) {
    final n = length(maxLength); _need(n);
    final v = utf8.decode(_bytes.sublist(_offset, _offset + n));
    _offset += n; return v;
  }
  void _need(int n) {
    if (_offset + n > _bytes.length) throw const PostcardException('Input buffer too small');
  }
  void finish() {
    if (_offset != _bytes.length) throw const PostcardException('Trailing input bytes');
  }
}

"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{BindingsRegistry, StructType};

    #[test]
    fn generates_typed_model_and_codec() {
        let mut value = StructType::new();
        value.register_field::<u16>("value");
        value.register_field::<String>("label");
        let mut registry = BindingsRegistry::default();
        registry.register_struct_binding("Reading", "", value);
        registry.register_unit_struct_binding(
            "Status",
            "test_crate::device",
            crate::registry::UnitStructType::new(),
        );
        let src = generate(registry.into_entries(), &GenerationSettings::enable_all());
        assert!(src.contains("class Reading"));
        assert!(src.contains("class device_Status"));
        assert!(src.contains("final int value;"));
        assert!(src.contains("Uint8List serialize(Object value)"));
        assert!(src.contains("T deserialize<T>(Uint8List bytes)"));
    }
}
