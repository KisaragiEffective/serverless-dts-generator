# serverless-dts-generator
[serverless](https://www.npmjs.com/package/serverless)の[`serverless.yml`](https://www.serverless.com/framework/docs/providers/aws/guide/serverless.yml)から[`*.d.ts`](https://www.typescriptlang.org/docs/handbook/declaration-files/templates/module-d-ts.html)を生成するプログラムです。

## このアプリケーションの使い方
### ダウンロード
1. [Releases](https://github.com/KisaragiEffective/serverless-dts-generator/releases)からダウンロードします。
2. 一番上にあるバージョンを見ます。
3. `<プラットフォーム>`は以下のとおりです。
  * Windows: x86_64-pc-windows-gnu
  * Linux: unknown-linux-musl
  * macOS: x86_64-apple-darwin
4. `<拡張子>`は次のとおりです。
  * Windows: `zip`
  * Linux: `tar.gz`または`tar.xz`
  * macOS: `zip`
5. `serverless-dts-generator_<バージョン>_<プラットフォーム>.<拡張子>`と`serverless-dts-generator_<バージョン>_<プラットフォーム>.<拡張子>.sha256sum`をダウンロードします。
6. (推奨) [ハッシュ値を検証](#ハッシュ値の検証)します。
7. `serverless-dts-generator_<バージョン>_<プラットフォーム>.<拡張子>`を展開します。
8. 使用を開始するためには、[コマンドライン](#コマンドライン)へ移動します。

#### ハッシュ値の検証
Windows:

1. <kbd>Win</kbd> + <kbd>R</kbd>キーを押して、`powershell`と入力し、<kbd>Enter</kbd>を押します。
2. 次のコマンドをコピーアンドペーストして<kbd>Enter</kbd>を押します。

```pwsh
$archive_file = "kisaragi-booth-utility_0.1.1_x86_64-pc-windows-gnu.zip"
$hash_file = $archive_file + ".sha256sum"
$actual_hash = if ($PSVersionTable.PSCompatibleVersions -contains [System.Version]::New(4, 0)) {
  $hash_obj = Get-FileHash $archive_file -Algorithm SHA256
  $hash_obj.Hash.ToLower() + " " + $(Split-Path $hash_obj.Path -leaf)
} else {
  # Get-FileHash is unsupported
  $hasher = [System.Security.Cryptography.SHA256]::Create()
  $io = New-Object System.IO.StreamReader $archive_file
  $hash_arr = $hasher.ComputeHash($io.BaseStream)
  $stream.Close()
  $hash = ""
  $hash_arr | %{ $hash += $_.ToString("x2") }
  $hash
}
$expected_hash = (type $hash_file) -join ""
if ($actual_hash -eq $expected_hash) {
  Write-Host "Hash OK"
} else {
  Write-Error "Hash Error: '$actual_hash' != '$expected_hash'"
}
```

3. `Hash OK`と表示された場合、検証が完了しています。

Linux/macOS:

1. お好みのPOSIX互換シェルを開きます。
2. 次のコードをコピーアンドペーストして実行します。macOSでは`sha256sum`を`gsha256sum`に変える必要があります。

```sh
#!/bin/sh
actual_hash=$(sha256sum kisaragi-booth-utility_0.1.1_x86_64-unknown-linux-musl.tar.gz)
expected_hash=$(cat kisaragi-booth-utility_0.1.1_x86_64-unknown-linux-musl.tar.gz.sha256sum)
if [ "$original_hash" -eq "$expected_hash" ]; then
    echo "Hash OK"
else
    echo "Hash Error: '$actual_hash' != '$expected_hash'" >&2
fi
```

3. `Hash OK`と表示された場合、検証が完了しています。

## コマンドライン
```
serverless-dts-generator [serverless.ymlのパス]
```

* `[serverless.ymlのパス]` - `serverless.yml`のパスを指定します。

## 注意点
* `*.d.ts`は`serverless.yml`などのスキーマに設定されたパスに応じて生成される場所が変わります。具体的には、`handler`プロパティに設定されたパスと同じディレクトリに、ファイル名と同名の`.d.ts`が生成されます。
    * 例：`handler`プロパティに`src/hoge/piyo.createPiyo`が設定されている場合、`createPiyo`という名前の`declare const`を生成し、`src/hoge/piyo.d.ts`にそれを書き込みます。

## ライセンス
Apache License, Version 2.0とMITライセンスのデュアルライセンスです。どちらかのライセンスを選択することができます。

各ライセンス文はそれぞれLICENSE-Apache-2.0.txtとLICENSE-MIT.txtに同梱してあります。
