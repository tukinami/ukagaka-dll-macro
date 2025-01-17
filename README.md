# ukagaka-dll-macro

[GitHub repository](https://github.com/tukinami/ukagaka-dll-macro)

## これは何?

デスクトップマスコット、「伺か」用DLLのためのマクロ集です。

伺かのDLLに使われる`load`と`loadu`、`request`、`unload`を定義するマクロがあります。

Dllへのパスを返す関数 `read_dll_path_string`と、`dll_main`featureが有効のときのみ使用可能な、DLLのエントリポイントである`DllMain`を定義するマクロ`define_dll_main`も定義しています。

### 使い方

以下のマクロを使用するための型や関数を定義してあるので、使用するときは、

```rust
use ukagaka_dll_macro::*;
```

とグロブで`use`してください。

また、以下に記載のないものについては、自分で`load`などを定義したいとき以外は、あまり触る必要はないと思います。

### `define_load`マクロ

関数`load`と`loadu`を定義します。

引数で、DLLへのパスである`&str`を受けとり、`bool`を返す関数名を渡してください。
内部でDLLへのパスを記録しています。(記録したパスは`read_dll_path_string`で呼び出せます)

v1.1.0より、関数名は省略不可になりました。

### `define_request`マクロ

関数`request`を定義します。

引数で、requestの内容である`&Vec<u8>`を受けとり、返答である`Vec<i8>`を返す関数名を渡してください。

### `define_unload`マクロ

関数`unload`を定義します。

引数で`bool`を返す関数名を渡してください(省略可)。

### `read_dll_path_string`関数

DLLへのパスを`Option<String>`で返します。

`define_load` が呼ばれていないと、`None`しか返しません。

### `define_dll_main`マクロ(`dll_main`feature有効時のみ)

関数`DllMain`を定義します。

引数は順番に、`DLL_PROCESS_ATTACH`時、`DLL_PROCESS_DETACH`時、`DLL_THREAD_ATTACH`時、`DLL_THREAD_DETACH`時の処理になります。
それぞれ省略可で、もし、途中を飛ばしたい場合、`()`を指定してください。それでその時点での処理はなくなります。
引数なしなら、何もしません。(v1.1.0より)

featureの`dll_main`が有効になっていないと使用できませんが、基本的な動作には必要ありません。

## 例

```rust
// lib.rs
use ukagaka_dll_macro::*;

// v1.1.0より、DLLのパスを引数にとるようになりました。
fn ukagaka_load(_path: &str) -> bool {
    // process with dll path
    true
}

fn ukagaka_request(_s: &[u8]) -> Vec<i8> {
    if let Some(_dll_path) = read_dll_path_string() {
        // process with dll path & s(contents of request).
    }
    // build response
    b"SAORI/1.0 200 OK\r\nResult:1\r\nCharset:UTF-8\r\n\r\n\0"
        .iter()
        .map(|v| *v as i8)
        .collect()
}

// v1.1.0より、`define_dll_main`マクロを呼ばなくても、
// `define_load`が呼ばれていれば、DLLのパスを記録するようになりました。
define_load!(ukagaka_load);
define_request!(ukagaka_request);
define_unload!();
```

## 使用ライブラリ

いずれも敬称略。ありがとうございます。

+ [winapi\_rs](https://github.com/retep998/winapi-rs) / Peter Atashian
+ [encoding](https://github.com/lifthrasiir/rust-encoding) / Kang Seonghoon

## ライセンス

MITにて配布いたします。

## 作成者

月波 清火 (tukinami seika)

[GitHub](https://github.com/tukinami)
