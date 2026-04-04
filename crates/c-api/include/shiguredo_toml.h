#ifndef SHIGUREDO_TOML_H
#define SHIGUREDO_TOML_H

/* Generated with cbindgen:0.29.2 */

#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>

/**
 * TOML バージョン。
 */
typedef enum TomlVersion {
  /**
   * TOML v1.0.0
   */
  TOML_VERSION_1_0 = 0,
  /**
   * TOML v1.1.0
   */
  TOML_VERSION_1_1 = 1,
} TomlVersion;

/**
 * TOML 操作のエラーコード。
 */
typedef enum TomlError {
  /**
   * 成功
   */
  TOML_OK = 0,
  /**
   * 解析エラー
   */
  TOML_ERROR_PARSE,
  /**
   * 直列化エラー
   */
  TOML_ERROR_SERIALIZE,
  /**
   * バリデーションエラー
   */
  TOML_ERROR_VALIDATE,
  /**
   * null ポインタが渡された
   */
  TOML_ERROR_NULL_POINTER,
  /**
   * 型の不一致
   */
  TOML_ERROR_TYPE_MISMATCH,
  /**
   * キーが見つからない
   */
  TOML_ERROR_KEY_NOT_FOUND,
  /**
   * インデックスが範囲外
   */
  TOML_ERROR_INDEX_OUT_OF_RANGE,
} TomlError;

/**
 * TOML 値の型を表す列挙型。
 */
typedef enum TomlValueKind {
  TOML_VALUE_STRING = 0,
  TOML_VALUE_INTEGER = 1,
  TOML_VALUE_FLOAT = 2,
  TOML_VALUE_BOOLEAN = 3,
  TOML_VALUE_DATETIME = 4,
  TOML_VALUE_ARRAY = 5,
  TOML_VALUE_TABLE = 6,
} TomlValueKind;

/**
 * 不透明な TOML テーブル型。パース結果のルートテーブルを保持する。
 */
typedef struct TomlTable TomlTable;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * ライブラリバージョン文字列を返す。
 *
 * 返されるポインタは静的領域を指し、解放不要。
 */
const char *toml_library_version(void);

/**
 * 最後のエラーメッセージを返す。
 *
 * エラーがない場合は空文字列のポインタを返す。
 * 返されるポインタは次のエラー発生まで有効。
 */
const char *toml_get_last_error(void);

/**
 * 最後の解析エラーのバイト位置を返す。
 *
 * 解析エラーでない場合は -1 を返す。
 */
int64_t toml_get_last_error_position(void);

/**
 * TOML 文字列をパースして TomlTable を返す。TOML v1.0.0 を使用する。
 *
 * 成功時は TomlTable へのポインタを返す。失敗時は null を返す。
 * 返された TomlTable は `toml_table_free` で解放する必要がある。
 */
struct TomlTable *toml_parse(const char *input);

/**
 * TOML 文字列を指定バージョンでパースして TomlTable を返す。
 *
 * 成功時は TomlTable へのポインタを返す。失敗時は null を返す。
 * 返された TomlTable は `toml_table_free` で解放する必要がある。
 */
struct TomlTable *toml_parse_with_version(const char *input, enum TomlVersion version);

/**
 * TomlTable を解放する。null ポインタは無視する。
 */
void toml_table_free(struct TomlTable *table);

/**
 * テーブル内のキー数を返す。
 */
uintptr_t toml_table_len(const struct TomlTable *table);

/**
 * テーブルのキー一覧から指定インデックスのキーを返す。
 *
 * 範囲外の場合は null を返す。
 * 返されるポインタは TomlTable が解放されるまで有効。
 */
const char *toml_table_key_at(struct TomlTable *table, uintptr_t index);

/**
 * テーブルにキーが存在するかどうかを返す。
 */
bool toml_table_contains_key(const struct TomlTable *table, const char *key);

/**
 * テーブルから指定キーの値の型を取得する。
 *
 * キーが存在しない場合は TOML_ERROR_KEY_NOT_FOUND を返す。
 */
enum TomlError toml_table_get_kind(const struct TomlTable *table,
                                   const char *key,
                                   enum TomlValueKind *out_kind);

/**
 * テーブルから文字列値を取得する。
 *
 * 値が文字列でない場合は TOML_ERROR_TYPE_MISMATCH を返す。
 * 返される文字列は null 終端ではない。out_len でバイト長を取得すること。
 * 返されるポインタは TomlTable が解放されるまで有効。
 */
enum TomlError toml_table_get_string(const struct TomlTable *table,
                                     const char *key,
                                     const char **out_value,
                                     uintptr_t *out_len);

/**
 * テーブルから整数値を取得する。
 */
enum TomlError toml_table_get_integer(const struct TomlTable *table,
                                      const char *key,
                                      int64_t *out_value);

/**
 * テーブルから浮動小数点値を取得する。
 */
enum TomlError toml_table_get_float(const struct TomlTable *table,
                                    const char *key,
                                    double *out_value);

/**
 * テーブルからブール値を取得する。
 */
enum TomlError toml_table_get_bool(const struct TomlTable *table, const char *key, bool *out_value);

/**
 * テーブルから日時の文字列表現を取得する。
 *
 * 返される文字列は null 終端。TomlTable が解放されるまで有効。
 */
enum TomlError toml_table_get_datetime(struct TomlTable *table,
                                       const char *key,
                                       const char **out_value);

/**
 * テーブルからサブテーブルのキー数を取得する。
 *
 * 値がテーブルでない場合は TOML_ERROR_TYPE_MISMATCH を返す。
 */
enum TomlError toml_table_get_subtable_len(const struct TomlTable *table,
                                           const char *key,
                                           uintptr_t *out_len);

/**
 * テーブルから配列の要素数を取得する。
 *
 * 値が配列でない場合は TOML_ERROR_TYPE_MISMATCH を返す。
 */
enum TomlError toml_table_get_array_len(const struct TomlTable *table,
                                        const char *key,
                                        uintptr_t *out_len);

/**
 * ドット区切りパスで値の型を取得する。
 *
 * パスは "servers.alpha.port" のような形式。
 * 配列インデックスは "items[0]" のような形式。
 */
enum TomlError toml_table_get_kind_by_path(const struct TomlTable *table,
                                           const char *path,
                                           enum TomlValueKind *out_kind);

/**
 * ドット区切りパスで文字列値を取得する。
 */
enum TomlError toml_table_get_string_by_path(const struct TomlTable *table,
                                             const char *path,
                                             const char **out_value,
                                             uintptr_t *out_len);

/**
 * ドット区切りパスで整数値を取得する。
 */
enum TomlError toml_table_get_integer_by_path(const struct TomlTable *table,
                                              const char *path,
                                              int64_t *out_value);

/**
 * ドット区切りパスで浮動小数点値を取得する。
 */
enum TomlError toml_table_get_float_by_path(const struct TomlTable *table,
                                            const char *path,
                                            double *out_value);

/**
 * ドット区切りパスでブール値を取得する。
 */
enum TomlError toml_table_get_bool_by_path(const struct TomlTable *table,
                                           const char *path,
                                           bool *out_value);

/**
 * TomlTable を TOML 文字列にシリアライズする。
 *
 * 成功時は null 終端の文字列ポインタを返す。失敗時は null を返す。
 * 返されるポインタは TomlTable が解放されるか次のシリアライズ呼び出しまで有効。
 */
const char *toml_serialize(struct TomlTable *table);

/**
 * TomlTable を整形済み TOML 文字列にシリアライズする。
 *
 * 成功時は null 終端の文字列ポインタを返す。失敗時は null を返す。
 * 返されるポインタは TomlTable が解放されるか次のシリアライズ呼び出しまで有効。
 */
const char *toml_serialize_pretty(struct TomlTable *table);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* SHIGUREDO_TOML_H */
