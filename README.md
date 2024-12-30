# ukagaka-dll-macro

[GitHub repository](https://github.com/tukinami/ukagaka-dll-macro)

## 注意

このライブラリは現在(2024-12-30)開発中です。

思わぬバグや仕様変更の可能性があります。

## これは何?

デスクトップマスコット、「伺か」用DLLのためのマクロ集です。

伺かのDLLに使われる`load`、`request`、`unload`と、DLLのエントリポイントである`DllMain`を定義するマクロがあります。

おまけで、Dllへのパスを返す関数 `read_dll_path_string` も定義しています。

### 使い方

以下のマクロを使用するための型や関数を定義してあるので、使用するときは、

```rust
use ukagaka_dll_macro::*;
```

とグロブで`use`してください。

また、以下に記載のないものについては、自分で`load`などを定義したいとき以外は、あまり触る必要はないと思います。

### `define_dll_main`マクロ

関数`DllMain`を定義します。

引数1つなら、`PROCESS_DETACH`時、2つなら、2つめが`PROCESS_ATTACH`時にそれぞれ処理が走ります。
引数なしなら、以下の動作のみになります。
内部で `register_dll_path_string` を呼んで、DLLへのパスを記録しています。

### `define_load`マクロ

関数`load`を定義します。

引数で`bool`を返す関数名を渡してください。

### `define_request`マクロ

関数`request`を定義します。

引数で、requestの内容である`&Vec<u8>`を受けとり、返答である`Vec<i8>`を返す関数名を渡してください。

### `define_unload`マクロ

関数`unload`を定義します。

引数で`bool`を返す関数名を渡してください。

### `read_dll_path_string`関数

DLLへのパスを`Option<String>`で返します。

`register_dll_path_string` が呼ばれていないと、`None`しか返しません。

`register_dll_path_string`は`define_dll_macro`で呼ばれているので、通常`register~`を手動で呼ぶ必要はありません。

## 例

```rust
use ukagaka_dll_macro::*;

fn ukagaka_load() -> bool {
    true
}

fn ukagaka_request(_s: &Vec<u8>) -> Vec<i8> {
    if let Some(_dll_path) = read_dll_path_string() {
        // process with dll path & s(contents of request).
    }
    // build response
    b"SAORI/1.0 200 OK\r\nResult:1\r\nCharset:UTF-8\r\n\r\n\0"
        .iter()
        .map(|v| *v as i8)
        .collect()
}

fn ukagaka_unload() -> bool {
    true
}

define_dll_main!();
define_load!(ukagaka_load);
define_request!(ukagaka_request);
define_unload!(ukagaka_unload);
```

## 使用ライブラリ

いずれも敬称略。ありがとうございます。

+ [winapi\_rs](https://github.com/retep998/winapi-rs) / Peter Atashian

## ライセンス

MITにて配布いたします。

## 作成者

月波 清火 (tukinami seika)

[GitHub](https://github.com/tukinami)
