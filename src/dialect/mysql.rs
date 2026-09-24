use gpui_kit::SharedString;

use crate::{
    dialect::Dialect, model::{AttrKind, AttrSpec, Column, ColumnType, ColumnTypeCategory, ColumnTypeParam, ColumnTypeSpec},
};

pub struct MySqlDialect {}

impl MySqlDialect {
    fn charsets() -> &'static [&'static str] {
        &[
            "armscii8", "ascii", "big5", "binary", "cp1250", "cp1251", "cp1256", "cp1257", "cp850",
            "cp852", "cp866", "cp932", "dec8", "eucjpms", "euckr", "gb18030", "gb2312", "gbk",
            "geostd8", "greek", "hebrew", "hp8", "keybcs2", "koi8r", "koi8u", "latin1", "latin2",
            "latin5", "latin7", "macce", "macroman", "sjis", "swe7", "tis620", "ucs2", "ujis",
            "utf16", "utf16le", "utf32", "utf8mb3", "utf8mb4",
        ]
    }

    fn charsets_owned() -> Vec<SharedString> {
        Self::charsets()
            .iter()
            .map(|s| SharedString::new_static(*s))
            .collect::<Vec<_>>()
    }

    fn collations() -> &'static [&'static str] {
        &[
            "armscii8_bin",
            "armscii8_general_ci",
            "ascii_bin",
            "ascii_general_ci",
            "big5_bin",
            "big5_chinese_ci",
            "binary",
            "cp1250_bin",
            "cp1250_croatian_ci",
            "cp1250_czech_cs",
            "cp1250_general_ci",
            "cp1250_polish_ci",
            "cp1251_bin",
            "cp1251_bulgarian_ci",
            "cp1251_general_ci",
            "cp1251_general_cs",
            "cp1251_ukrainian_ci",
            "cp1256_bin",
            "cp1256_general_ci",
            "cp1257_bin",
            "cp1257_general_ci",
            "cp1257_lithuanian_ci",
            "cp850_bin",
            "cp850_general_ci",
            "cp852_bin",
            "cp852_general_ci",
            "cp866_bin",
            "cp866_general_ci",
            "cp932_bin",
            "cp932_japanese_ci",
            "dec8_bin",
            "dec8_swedish_ci",
            "eucjpms_bin",
            "eucjpms_japanese_ci",
            "euckr_bin",
            "euckr_korean_ci",
            "gb18030_bin",
            "gb18030_chinese_ci",
            "gb18030_unicode_520_ci",
            "gb2312_bin",
            "gb2312_chinese_ci",
            "gbk_bin",
            "gbk_chinese_ci",
            "geostd8_bin",
            "geostd8_general_ci",
            "greek_bin",
            "greek_general_ci",
            "hebrew_bin",
            "hebrew_general_ci",
            "hp8_bin",
            "hp8_english_ci",
            "keybcs2_bin",
            "keybcs2_general_ci",
            "koi8r_bin",
            "koi8r_general_ci",
            "koi8u_bin",
            "koi8u_general_ci",
            "latin1_bin",
            "latin1_danish_ci",
            "latin1_general_ci",
            "latin1_general_cs",
            "latin1_german1_ci",
            "latin1_german2_ci",
            "latin1_spanish_ci",
            "latin1_swedish_ci",
            "latin2_bin",
            "latin2_croatian_ci",
            "latin2_czech_cs",
            "latin2_general_ci",
            "latin2_hungarian_ci",
            "latin5_bin",
            "latin5_turkish_ci",
            "latin7_bin",
            "latin7_estonian_cs",
            "latin7_general_ci",
            "latin7_general_cs",
            "macce_bin",
            "macce_general_ci",
            "macroman_bin",
            "macroman_general_ci",
            "sjis_bin",
            "sjis_japanese_ci",
            "swe7_bin",
            "swe7_swedish_ci",
            "tis620_bin",
            "tis620_thai_ci",
            "ucs2_bin",
            "ucs2_croatian_ci",
            "ucs2_czech_ci",
            "ucs2_danish_ci",
            "ucs2_esperanto_ci",
            "ucs2_estonian_ci",
            "ucs2_general_ci",
            "ucs2_german2_ci",
            "ucs2_hungarian_ci",
            "ucs2_icelandic_ci",
            "ucs2_latvian_ci",
            "ucs2_lithuanian_ci",
            "ucs2_persian_ci",
            "ucs2_polish_ci",
            "ucs2_roman_ci",
            "ucs2_romanian_ci",
            "ucs2_sinhala_ci",
            "ucs2_slovak_ci",
            "ucs2_slovenian_ci",
            "ucs2_spanish2_ci",
            "ucs2_spanish_ci",
            "ucs2_swedish_ci",
            "ucs2_turkish_ci",
            "ucs2_unicode_ci",
            "ucs2_unicode_520_ci",
            "ucs2_vietnamese_ci",
            "ujis_bin",
            "ujis_japanese_ci",
            "utf16_bin",
            "utf16_croatian_ci",
            "utf16_czech_ci",
            "utf16_danish_ci",
            "utf16_esperanto_ci",
            "utf16_estonian_ci",
            "utf16_general_ci",
            "utf16_german2_ci",
            "utf16_hungarian_ci",
            "utf16_icelandic_ci",
            "utf16_latvian_ci",
            "utf16_lithuanian_ci",
            "utf16_persian_ci",
            "utf16_polish_ci",
            "utf16_roman_ci",
            "utf16_romanian_ci",
            "utf16_sinhala_ci",
            "utf16_slovak_ci",
            "utf16_slovenian_ci",
            "utf16_spanish2_ci",
            "utf16_spanish_ci",
            "utf16_swedish_ci",
            "utf16_turkish_ci",
            "utf16_unicode_ci",
            "utf16_unicode_520_ci",
            "utf16_vietnamese_ci",
            "utf16le_bin",
            "utf16le_general_ci",
            "utf32_bin",
            "utf32_croatian_ci",
            "utf32_czech_ci",
            "utf32_danish_ci",
            "utf32_esperanto_ci",
            "utf32_estonian_ci",
            "utf32_general_ci",
            "utf32_german2_ci",
            "utf32_hungarian_ci",
            "utf32_icelandic_ci",
            "utf32_latvian_ci",
            "utf32_lithuanian_ci",
            "utf32_persian_ci",
            "utf32_polish_ci",
            "utf32_roman_ci",
            "utf32_romanian_ci",
            "utf32_sinhala_ci",
            "utf32_slovak_ci",
            "utf32_slovenian_ci",
            "utf32_spanish2_ci",
            "utf32_spanish_ci",
            "utf32_swedish_ci",
            "utf32_turkish_ci",
            "utf32_unicode_ci",
            "utf32_unicode_520_ci",
            "utf32_vietnamese_ci",
            "utf8mb3_bin",
            "utf8mb3_croatian_ci",
            "utf8mb3_czech_ci",
            "utf8mb3_danish_ci",
            "utf8mb3_esperanto_ci",
            "utf8mb3_estonian_ci",
            "utf8mb3_general_ci",
            "utf8mb3_german2_ci",
            "utf8mb3_hungarian_ci",
            "utf8mb3_icelandic_ci",
            "utf8mb3_latvian_ci",
            "utf8mb3_lithuanian_ci",
            "utf8mb3_persian_ci",
            "utf8mb3_polish_ci",
            "utf8mb3_roman_ci",
            "utf8mb3_romanian_ci",
            "utf8mb3_sinhala_ci",
            "utf8mb3_slovak_ci",
            "utf8mb3_slovenian_ci",
            "utf8mb3_spanish2_ci",
            "utf8mb3_spanish_ci",
            "utf8mb3_swedish_ci",
            "utf8mb3_turkish_ci",
            "utf8mb3_unicode_ci",
            "utf8mb3_unicode_520_ci",
            "utf8mb3_vietnamese_ci",
            "utf8mb4_0900_ai_ci",
            "utf8mb4_0900_as_ci",
            "utf8mb4_0900_as_cs",
            "utf8mb4_0900_bin",
            "utf8mb4_bin",
            "utf8mb4_general_ci",
            "utf8mb4_unicode_ci",
            "utf8mb4_unicode_520_ci",
        ]
    }

    fn collations_owned() -> Vec<SharedString> {
        Self::collations()
            .iter()
            .map(|s| SharedString::from(*s))
            .collect::<Vec<_>>()
    }
}

impl Dialect for MySqlDialect {
    fn name() -> &'static str {
        "mysql"
    }

    fn database_attributes() -> Vec<AttrSpec> {
        vec![
            AttrSpec {
                key: "charset",
                label: "Character set",
                kind: AttrKind::Select {
                    options: MySqlDialect::charsets_owned(),
                },
                required: false,
            },
            AttrSpec {
                key: "collation",
                label: "Collation",
                kind: AttrKind::Select {
                    options: MySqlDialect::collations_owned(),
                },
                required: false,
            },
        ]
    }

    fn table_attributes() -> Vec<AttrSpec> {
        let mut items = Self::database_attributes();
        items.push(AttrSpec {
            key: "comment",
            label: "Comment",
            kind: AttrKind::Text {
                default: None,
                multiple_line: true,
            },
            required: false,
        });

        items
    }

    fn column_types() -> Vec<ColumnTypeSpec> {
        use ColumnTypeCategory::{DateTime, Number, Spatial, String};

        vec![
            // ---------- 整数（8.x 里只有整数类型支持 AUTO_INCREMENT） ----------
            ColumnTypeSpec { name: "TINYINT",   category: Number, params: &[ColumnTypeParam::Unsigned], supports_auto_increment: true  },
            ColumnTypeSpec { name: "SMALLINT",  category: Number, params: &[ColumnTypeParam::Unsigned], supports_auto_increment: true  },
            ColumnTypeSpec { name: "MEDIUMINT", category: Number, params: &[ColumnTypeParam::Unsigned], supports_auto_increment: true  },
            ColumnTypeSpec { name: "INT",   category: Number, params: &[ColumnTypeParam::Unsigned], supports_auto_increment: true  },
            ColumnTypeSpec { name: "BIGINT",    category: Number, params: &[ColumnTypeParam::Unsigned], supports_auto_increment: true  },

            // ---------- 布尔（其实是 TINYINT(1) 的别名） ----------
            ColumnTypeSpec { name: "BOOL",      category: Number, params: &[], supports_auto_increment: false },

            // ---------- 定点 / 浮点 ----------
            ColumnTypeSpec { name: "DECIMAL",   category: Number, params: &[ColumnTypeParam::Precision, ColumnTypeParam::Scale], supports_auto_increment: false },
            ColumnTypeSpec { name: "FLOAT",     category: Number, params: &[ColumnTypeParam::Precision], supports_auto_increment: false },
            ColumnTypeSpec { name: "DOUBLE",    category: Number, params: &[ColumnTypeParam::Precision], supports_auto_increment: false },
            ColumnTypeSpec { name: "REAL",      category: Number, params: &[ColumnTypeParam::Precision, ColumnTypeParam::Scale], supports_auto_increment: false },

            // ---------- 位类型 ----------
            ColumnTypeSpec { name: "BIT",       category: Number, params: &[ColumnTypeParam::Length], supports_auto_increment: false },

            // ---------- 日期时间 ----------
            ColumnTypeSpec { name: "DATE",      category: DateTime, params: &[], supports_auto_increment: false },
            ColumnTypeSpec { name: "DATETIME",  category: DateTime, params: &[ColumnTypeParam::Precision], supports_auto_increment: false },
            ColumnTypeSpec { name: "TIMESTAMP", category: DateTime, params: &[ColumnTypeParam::Precision], supports_auto_increment: false },
            ColumnTypeSpec { name: "TIME",      category: DateTime, params: &[ColumnTypeParam::Precision], supports_auto_increment: false },
            ColumnTypeSpec { name: "YEAR",      category: DateTime, params: &[], supports_auto_increment: false },

            // ---------- 字符串 / 二进制 ----------
            ColumnTypeSpec { name: "CHAR",      category: String, params: &[ColumnTypeParam::Length], supports_auto_increment: false },
            ColumnTypeSpec { name: "VARCHAR",   category: String, params: &[ColumnTypeParam::Length], supports_auto_increment: false },
            ColumnTypeSpec { name: "BINARY",    category: String, params: &[ColumnTypeParam::Length], supports_auto_increment: false },
            ColumnTypeSpec { name: "VARBINARY", category: String, params: &[ColumnTypeParam::Length], supports_auto_increment: false },

            ColumnTypeSpec { name: "TINYTEXT",   category: String, params: &[], supports_auto_increment: false },
            ColumnTypeSpec { name: "TEXT",       category: String, params: &[], supports_auto_increment: false },
            ColumnTypeSpec { name: "MEDIUMTEXT", category: String, params: &[], supports_auto_increment: false },
            ColumnTypeSpec { name: "LONGTEXT",   category: String, params: &[], supports_auto_increment: false },

            ColumnTypeSpec { name: "TINYBLOB",   category: String, params: &[], supports_auto_increment: false },
            ColumnTypeSpec { name: "BLOB",       category: String, params: &[], supports_auto_increment: false },
            ColumnTypeSpec { name: "MEDIUMBLOB", category: String, params: &[], supports_auto_increment: false },
            ColumnTypeSpec { name: "LONGBLOB",   category: String, params: &[], supports_auto_increment: false },

            ColumnTypeSpec { name: "ENUM",      category: String, params: &[ColumnTypeParam::EnumValues], supports_auto_increment: false },
            ColumnTypeSpec { name: "SET",       category: String, params: &[ColumnTypeParam::EnumValues], supports_auto_increment: false },

            // 你现有 category 里没有 Json，暂时归到 String
            ColumnTypeSpec { name: "JSON",      category: String, params: &[], supports_auto_increment: false },

            // ---------- 空间类型 ----------
            ColumnTypeSpec { name: "GEOMETRY",           category: Spatial, params: &[], supports_auto_increment: false },
            ColumnTypeSpec { name: "POINT",              category: Spatial, params: &[], supports_auto_increment: false },
            ColumnTypeSpec { name: "LINESTRING",         category: Spatial, params: &[], supports_auto_increment: false },
            ColumnTypeSpec { name: "POLYGON",            category: Spatial, params: &[], supports_auto_increment: false },
            ColumnTypeSpec { name: "MULTIPOINT",         category: Spatial, params: &[], supports_auto_increment: false },
            ColumnTypeSpec { name: "MULTILINESTRING",    category: Spatial, params: &[], supports_auto_increment: false },
            ColumnTypeSpec { name: "MULTIPOLYGON",       category: Spatial, params: &[], supports_auto_increment: false },
            ColumnTypeSpec { name: "GEOMETRYCOLLECTION", category: Spatial, params: &[], supports_auto_increment: false },
            ColumnTypeSpec { name: "GEOMCOLLECTION",     category: Spatial, params: &[], supports_auto_increment: false },
        ]
    }

    fn default_pk_column() -> Option<Column> {
        Some(Column {
            column_type: ColumnType {
                name: "INT".into(),
                length: None,
                precision: None,
                scale: None,
                unsigned: false,
                values: None,
            },
            auto_increment: true,
            ..Column::new("id")
        })
    }
}
