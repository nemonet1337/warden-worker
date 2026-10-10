# Warden: Cloudflare Workers 向け Bitwarden 互換サーバー

[English](README.md) | **日本語**

[![Powered by Cloudflare](https://img.shields.io/badge/Powered%20by-Cloudflare-F38020?logo=cloudflare&logoColor=white)](https://www.cloudflare.com/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Deploy to Cloudflare Workers](https://img.shields.io/badge/Deploy%20to-Cloudflare%20Workers-orange?logo=cloudflare&logoColor=white)](https://workers.cloudflare.com/)

このプロジェクトは、セルフホスト可能な Bitwarden 互換サーバーを Cloudflare Workers に無料でデプロイできるようにするものです。サーバー管理やランニングコストを気にせず、「デプロイして放置」できる低メンテナンスな設計を目指しています。

## なぜもう一つ Bitwarden サーバー？

[Vaultwarden](https://github.com/dani-garcia/vaultwarden) のようなプロジェクトは優れたセルフホスト環境を提供しますが、それでもサーバーや VPS の管理と設計が必要です。これは手間がかかり、サーバー代の支払いを忘れるとパスワードにアクセスできなくなる可能性があります。

Warden は Cloudflare Workers エコシステムを活用してこの問題を解決します。Cloudflare Worker に Warden をデプロイし、ストレージに Cloudflare D1 を使うことで、完全に無料のサーバーレス・低メンテナンスな Bitwarden サーバーを実現できます。

## 機能

* **コア vault 機能:** 暗号化されたデータ（cipher）とフォルダの作成・読み取り・更新・削除。
* **ファイル添付:** 添付ファイル用に Cloudflare KV または R2 をオプションで利用可能。
* **ウェブサイトアイコン:** ログイン先サイトのファビコンをオンデマンドで取得し R2（または KV）にキャッシュ。
* **Bitwarden Send:** リンク経由で暗号化されたテキストやファイルを共有。
* **デバイス管理:** 有効なセッションの確認と失効。
* **ライブ同期とプッシュ通知:** WebSocket とモバイルプッシュによる vault のリアルタイム更新。
* **TOTP 対応:** Time-based One-Time Password の保存と生成。
* **Bitwarden 互換:** 公式 Bitwarden クライアントで動作。
* **無料でホスティング:** Cloudflare の無料枠で動作。
* **低メンテナンス:** 一度デプロイすれば放置して問題ありません。
* **安全性:** 暗号化されたデータは自分の Cloudflare D1 データベースに保存されます。
* **簡単デプロイ:** Wrangler CLI で数分で起動できます。

### 添付ファイル対応

Warden はファイル添付の保存先として **Cloudflare KV** または **Cloudflare R2** のどちらかを利用できます。

| 機能 | KV | R2 |
|---------|----|----|  
| 最大ファイルサイズ | **25 MB**（ハードリミット） | 100 MB（Workers のリクエストボディサイズ制限による） |
| クレジットカード | **不要** | 必要 |
| ストリーミング I/O | 可 | 可 |

**保存先の選択:** R2 が優先されます。R2 が設定されている場合は R2 が使われ、設定がなければ KV が使われます。

詳細な設定は[デプロイガイド](docs/deployment.md) を参照してください。R2 は追加料金が発生する場合があります。[Cloudflare R2 の料金](https://developers.cloudflare.com/r2/pricing/) を確認してください。

### Bitwarden Send

- **テキスト Send:** デフォルトで有効。追加設定は不要です。
- **ファイル Send:** 添付ファイル（[attachments](#添付ファイル対応)）と同様に、保存先として KV または R2 が必要です。

> [!NOTE]
> D1 の 1 行あたりのサイズ制限が 2 MB であるため、テキスト Send の最大サイズはおよそ **1.8 MiB** です。また `/api/sync` エンドポイントは現在のユーザーの全ての Send をレスポンスに直列化します。Send の数が多い、またはテキスト Send が非常に大きい場合、CPU 時間とレスポンスサイズが大幅に増加します。

## 現在のステータス

**このプロジェクトはまだ機能が完成していません**、~~完成することもないかもしれません~~。個人用途の vault としてのコア機能（TOTP を含む）には対応していますが、以下の機能は **未対応** です。

* 共有
* 2FA ログイン（TOTP を除く）
* 緊急アクセス
* 管理操作
* 組織
* その他 Bitwarden 高度な機能

これらを実装する計画は当面ありません。このプロジェクトの主な目標は、シンプルで無料・低メンテナンスな個人用パスワードマネージャーを提供することです。

## 対応状況

* **ブラウザ拡張:** Chrome、Firefox、Safari など（Chrome の 2026.3.0 で動作確認済み）
* **Android アプリ:** 公式 Bitwarden Android アプリ（2026.4.0 で動作確認済み）
* **iOS アプリ:** 公式 Bitwarden iOS アプリ（2026.4.0 で動作確認済み）

## デモ

デモインスタンスは [warden.qqnt.de](https://warden.qqnt.de) で利用できます。

`@warden-worker.demo` で終わるメールアドレスで新規アカウントを登録できます（メールアドレスの検証は不要です）。

デモインスタンスの利用をやめる場合は、他の人のためにアカウントを削除してください。

デモはレート制限に達して Cloudflare から無効化される可能性があるため、自分のインスタンスをデプロイすることを強くお勧めします。

## はじめに

- デプロイ方法を選択: [CLI デプロイ](docs/deployment.md#cli-deployment) または [GitHub Actions デプロイ](docs/deployment.md#cicd-deployment-with-github-actions)
- デプロイ手順に従ってシークレットと任意の添付ファイル設定を行ってください
- Bitwarden クライアントに Worker の URL を設定

## フロントエンド（Web Vault）

フロントエンドは [Cloudflare Workers Static Assets](https://developers.cloudflare.com/workers/static-assets/) を使って Worker にバンドルされます。Cloudflare のビルドでは、**固定バージョン**の [bw_web_builds](https://github.com/dani-garcia/bw_web_builds)（Vaultwarden の web vault）リリース（デフォルト: `v2026.7.0`）をダウンロードし、バックエンドと一緒にデプロイします。ビルド変数 `BW_WEB_VERSION` でバージョンを固定でき、`latest` にすると上流に追従します。

**仕組み:**

- 静的ファイル（HTML、CSS、JS）は Cloudflare のエッジネットワークから直接配信されます。
- API リクエスト（`/api/*`、`/identity/*`）は Rust Worker にルーティングされます。
- 別途 Pages デプロイやドメイン設定は不要です。
- リポジトリは Cloudflare にリンクされており、push されると自動的にビルド・デプロイされます。`scripts/cf-build.sh`（Workers Builds の **Build** コマンド）が rustup で Rust ツールチェーンを準備し、WASM バックエンドをコンパイルし、フロントエンドをダウンロード（`bw_web_builds`、`BW_WEB_VERSION` でバージョン固定、デフォルト `v2026.7.0`）します。

**UI 上書き（任意）:**

- このプロジェクトには `public/css/` に「軽量セルフホスト」向けの小さな UI 調整が同梱されています。
- CI/CD（および任意でローカル）では `bw_web_builds` を展開した後にこれらを適用します。
  - `mkdir -p public/web-vault/css/ && cp public/css/vaultwarden.css public/web-vault/css/`

> [!NOTE]
> フロントエンドを別々にデプロイしていた環境から移行する場合、以前 Cloudflare Pages にデプロイしていた `warden-frontend` Pages プロジェクトを削除し、Worker のルータを再設定できます。フロントエンドは Worker にバンドルされるようになったため、別途デプロイは不要です。

> [!WARNING]
> web vault のフロントエンドは Vaultwarden 由来であるため多くの高度な UI が表示されますが、そのほとんどは機能しません。[現在のステータス](#現在のステータス) を参照してください。

## カスタムドメインの設定（任意）

デフォルトの `*.workers.dev` ドメインの代わりにカスタムドメインを使いたい場合は、以下の手順に従ってください。

### ステップ 1: DNS レコードを追加

1. [Cloudflare ダッシュボード](https://dash.cloudflare.com/) にログイン
2. ドメインを選択（例: `example.com`）
3. **DNS** → **レコード** を開く
4. **レコードを追加** をクリック:
   - **タイプ:** `A`（IPv6 の場合は `AAAA`）
   - **名前:** サブドメイン（例: `vault.example.com` にする場合は `vault`）
   - **IPv4 アドレス:** `192.0.2.1`（プレースホルダ。実際のルーティングは Worker が行います）
   - **プロキシ:** **プロキシ済み**（オレンジの雲のアイコン - 必須です）
   - **TTL:** 自動
5. **保存** をクリック

> [!IMPORTANT]
> **プロキシは「プロキシ済み」**（オレンジの雲）である必要があります。「DNS のみ」（灰色の雲）の場合、Worker のルートは機能しません。

### ステップ 2: Worker ルートを追加

1. **Workers & Pages** → 該当の `warden-worker` を選択
2. **設定** → **ドメインとルート** をクリック
3. **追加** → **ルート** をクリック
4. ルートを設定:
   - **ルート:** `vault.example.com/*`（実際のドメインに置き換えてください）
   - **ゾーン:** ドメインのゾーンを選択
   - **Worker:** `warden-worker`
5. **ルートを追加** をクリック

## 組み込みレート制限

このプロジェクトには [Cloudflare の Rate Limiting API](https://developers.cloudflare.com/workers/runtime-apis/bindings/rate-limit/) を使ったレート制限が組み込まれており、機密性の高いエンドポイントが保護されています。

| エンドポイント | レート制限 | キー種別 | 目的 |
|----------|------------|----------|---------|
| `/identity/connect/token`（パスワードグラント） | 5 リクエスト/分（メールアドレスごと）・ 5 リクエスト/分（IP ごと） | メールアドレス + IP アドレス | パスワード総当たり・クレデンシャルスタッフィング対策 |
| `/identity/connect/token`（Send アクセス） | 15 リクエスト/分 | IP アドレス | 送信済み Send のパスワード総当たり対策 |
| `/api/sends/access/*`（パスワード保護） | 10 リクエスト/分 | IP アドレス | 送信済み Send のパスワード総当たり対策 |
| `/api/accounts/register` | 5 リクエスト/分 | IP アドレス | 大量登録とメールアドレス列挙対策 |
| `/api/accounts/prelogin` | 5 リクエスト/分 | IP アドレス | メールアドレス列挙対策 |

レート制限の設定は `wrangler.toml` で変更できます。

```toml
[[ratelimits]]
name = "LOGIN_RATE_LIMITER"
namespace_id = "1001"
# limit（リクエスト数）と period（10 または 60 秒）を調整
simple = { limit = 5, period = 60 }

[[ratelimits]]
name = "SEND_ACCESS_RATE_LIMITER"
namespace_id = "1003"
# 公開 Send のパスワード確認のため緩やかな IP 制限を使用
simple = { limit = 10, period = 60 }
```

> [!NOTE]
> `period` は `10` または `60` 秒のいずれかを指定する必要があります。詳細は [Cloudflare のドキュメント](https://developers.cloudflare.com/workers/runtime-apis/bindings/rate-limit/) を参照してください。

バインディングが存在しない場合（またはレートリミッターがエラーを返す場合）、リクエストはレート制限なしで処理されます（フェイルオープンによる緩やかな劣化）。保護を有効にするには `wrangler.toml` で `LOGIN_RATE_LIMITER` バインディングを設定してください。

## 設定

### CPU オフロード（Durable Objects 経由）

Cloudflare Workers の無料プランはリクエストごとの CPU 予算がごくわずかです。特に CPU 負荷が高いエンドポイントは次の 2 種類です。

- import エンドポイント: 大きな JSON ペイロード（通常 500kB〜1MB）のパース + バッチ挿入。
- 登録・ログイン・パスワード検証エンドポイント: パスワード検証のためのサーバー側 PBKDF2。

メインの Worker を高速に保ちながらこれらの操作をサポートするため、Warden は選択したエンドポイントを **Durable Objects（DO）** にオフロードできます。

- **Heavy DO**（`HEAVY_DO` バインディング）: Rust で `HeavyDo` として実装され（既存の axum ルータを再利用）、CPU 負荷の高いエンドポイントをより大きな CPU 予算で実行できます。

**有効化・無効化の方法**

CPU 負荷の高いエンドポイントがオフロードされるかどうかは、`wrangler.toml` に `HEAVY_DO` Durable Object バインディングが設定されているかどうかで決まります。

> [!NOTE]
> Durable Objects は無料プランでもリクエストあたり 30 秒という非常に大きな CPU 予算が使えます。[Cloudflare Durable Objects の制限](https://developers.cloudflare.com/durable-objects/platform/limits/) を参照してください。これを使って CPU 負荷の高いエンドポイントをオフロードします。

> Durable Objects には compute と storage の 2 種類の課金が発生する可能性があります。このプロジェクトでは storage は使用せず、無料プランで 1 日 10 万クエストと 13,000 GB-s の実行時間が利用可能で、ほとんどのユーザーにとって十分すぎるはずです。詳細は [Cloudflare Durable Objects の料金](https://developers.cloudflare.com/durable-objects/platform/pricing/) を参照してください。

> Durable Objects を無効化する場合、Cloudflare によるスロットリングを避けるため有料プランの契約が必要になることがあります。

### ライブ同期とプッシュ通知

Warden は 2 つの仕組みで vault データのライブ同期に対応しています: WebSocket プッシュ（デスクトップアプリとブラウザ拡張向け）とモバイルプッシュ通知（公式モバイルアプリ向け）です。

**WebSocket プッシュ（デスクトップとブラウザ拡張）**

この機能は Durable Objects によって実現され、`wrangler.toml` に `NOTIFY_DO` Durable Object バインディングが設定されている場合にデフォルトで有効になります。このバインディングとマイグレーションを削除すると、WebSocket 通知は無効になります。

**モバイルプッシュ通知**

Warden は Bitwarden のプッシュリレーサービス経由で、公式 Bitwarden モバイルアプリへのプッシュ通知に対応しています。

**設定手順**

1. [https://bitwarden.com/host/](https://bitwarden.com/host/) からインストール ID とキーを取得します。
2. 認証情報を Cloudflare ダッシュボードまたは `wrangler` CLI でシークレット `PUSH_INSTALLATION_ID` と `PUSH_INSTALLATION_KEY` として保存します。
3. `wrangler.toml` の `[vars]` または Cloudflare ダッシュボードで `PUSH_ENABLED` を `true` に設定してプッシュ通知を有効化します。

必要に応じて、既定のリレーエンドポイントを `PUSH_RELAY_URI` と `PUSH_IDENTITY_URI` でデフォルト（それぞれ `https://push.bitwarden.com`、`https://identity.bitwarden.com`）の値に上書きできます。

詳細な設定とトラブルシューティングは [Vaultwarden のプッシュ通知に関する wiki](https://github.com/dani-garcia/vaultwarden/wiki/Enabling-Mobile-Client-push-notification) を参照してください。

### その他の環境変数

環境変数は `wrangler.toml` の `[vars]` 配下に設定するか、Cloudflare ダッシュボードから設定します。

* **`BASE_URL`**（任意）
  - ファイルのアップロード・ダウンロード URL で使用されるベース URL を上書きします。
  - 例えば HTTPS プロトコル、ドメイン、443 以外のリバースプロキシを使う場合はポートを含めてください。パスは含めないでください。
  - 例: `https://vault.example.com` または `https://vault.example.com:8443`
  - 未設定の場合は受信リクエストから抽出します。
* **`PASSWORD_ITERATIONS`**（任意、既定値: `600000`）
  - サーバー側パスワードハッシュの PBKDF2 反復回数。
  - 最小値は 600000 です。
* **`TRASH_AUTO_DELETE_DAYS`**（任意、既定値: `30`）
  - 論理削除されたアイテムを完全削除するまでの日数。
  - `0` または負の値で無効化します。
* **`IMPORT_BATCH_SIZE`**（任意、既定値: `30`）
  - インポート処理のバッチサイズ。
  - `0` でバッチ処理を無効化します。
* **`DISABLE_USER_REGISTRATION`**（任意、既定値: `true`）
  - クライアント UI に登録ボタンを表示するかどうかを制御します（サーバーの動作は変わりません）。
* **`AUTHENTICATOR_DISABLE_TIME_DRIFT`**（任意、既定値: `false`）
  - `true` にすると TOTP 検証時の ±1 ステップの時間ずれを無効化します。
* **`ATTACHMENT_MAX_BYTES`**（任意）
  - 個別の添付ファイルの最大サイズ。
  - 例: 100MB の場合は `104857600`。
* **`ATTACHMENT_TOTAL_LIMIT_KB`**（任意）
  - ユーザーごとの添付ファイル合計容量の上限（KB）。
  - 例: 1GB の場合は `1048576`。
* **`LOG_LEVEL`**（任意、既定値: `info`）
  - ログの詳細度。`error`、`warn`、`info`、`debug`、`trace` のいずれか。
  - `debug` が機密データを出力する可能性があるため、本番では既定で `info` になります。
* **`SHOW_PASSWORD_HINT`**（任意、既定値: `false`）
  - `true` にすると `/api/accounts/password-hint` が保存済みのヒントを返します。
  - 既定では無効（非表示）で、メールアドレスを知っている他人にヒントを漏らしません。Vaultwarden 既定の動作に合わせています。
* **`CORS_ALLOW_ORIGINS`**（任意）
  - 許可する CORS オリジンのカンマ区切りリスト（例: `https://vault.example.com,https://example.com`）。
  - 未設定の場合は全てのオリジン（`*`）を許可します。Bearer トークン認証なので Cookie ベースの CSRF は攻撃経路になりませんが、本番では制限することを推奨します。
* **`ATTACHMENT_TTL_SECS`**（任意、既定値: `300`、最小値: `60`）
  - 添付ファイルのアップロード・ダウンロード URL の有効期限。
* **`SEND_TEXT_MAX_BYTES`**（任意、既定値: `1887436` ≈ 1.8 MiB）
  - テキスト Send のコンテンツ最大サイズ。D1 の 2 MB 行サイズ制限により制限されます。
* **`SEND_MAX_BYTES`**（任意、既定値: `104857600` = 100 MiB）
  - ファイル Send の最大サイズ。添付ファイルと同じ KV/R2 の制限を受けます。
* **`USER_SEND_LIMIT_KB`**（任意）
  - ユーザーごとの Send ファイル合計容量の上限（KB）。
* **`SEND_TTL_SECS`**（任意、既定値: `300`）
  - Send ファイルのアップロード・ダウンロード URL の有効期限。

### 定期タスク（Cron）

Worker は実行中に論理削除されたアイテムを削除する定期タスクを実行します。既定では毎日 03:00 UTC（`wrangler.toml` の `[triggers]` cron `"0 3 * * *"`）に実行されます。必要に応じて調整してください。cron 式の構文は [Cloudflare Cron Triggers のドキュメント](https://developers.cloudflare.com/workers/configuration/cron-triggers/) を参照してください。

## データベース運用

- **バックアップとリストア:** 自動バックアップと手動リストアの手順は [Database Backup & Restore](docs/db-backup-recovery.md#github-actions-backups) を参照してください。
- **Time Travel:** 特定時点に復旧する [D1 Time Travel](docs/db-backup-recovery.md#d1-time-travel-point-in-time-recovery) を参照してください。
- **グローバル等価ドメインのシード（任意）:** CLI デプロイと CI/CD でのシードについては [docs/deployment.md](docs/deployment.md) を参照してください。
- **D1 を使ったローカル開発:**
  - クイックスタート: `wrangler dev --persist`
  - フルスタック（Web Vault 込み）: デプロイ手順と同様にフロントエンドをダウンロードし `wrangler dev --persist` を実行
  - バックアップをローカルにインポート: `wrangler d1 execute vault1 --file=backup.sql`
  - ローカル DB の確認: `.wrangler/state/v3/d1/` 配下の SQLite ファイル

## D1 を使ったローカル開発

Wrangler を使って D1 対応の Worker をローカルで実行します。

**クイックスタート（API のみ）**

```bash
wrangler dev --persist
```

**フルスタック（Web Vault 込み）**

1. フロントエンドをダウンロードします（[デプロイ手順](docs/deployment.md#download-the-frontend-web-vault) を参照）。
2. ローカルで起動:

   ```bash
   wrangler dev --persist
   ```

3. `http://localhost:8787` で vault にアクセスできます。

**本番データを一時的に使う**

1. バックアップをダウンロードして復号します（[バックアップ手順](docs/db-backup-recovery.md#restoring-database-to-cloudflare-d1) を参照）。
2. `--remote` なしでローカルにインポート:

   ```bash
   wrangler d1 execute vault1 --file=backup.sql
   ```

3. `wrangler dev --persist` を起動し、クライアントから `http://localhost:8787` を参照するよう設定します。

**ローカル SQLite の確認**

```bash
ls .wrangler/state/v3/d1/
sqlite3 .wrangler/state/v3/d1/miniflare-D1DatabaseObject/*.sqlite
```

> [!NOTE]
> ローカル開発には Node.js と Wrangler が必要です。Worker は [workerd](https://github.com/cloudflare/workerd) によるシミュレート環境で実行されます。

## フォークの更新

GitHub フォーク経由でデプロイした場合、最新版への追従は簡単です。

1. **新しいリリースをウォッチ** — [このリポジトリ](https://github.com/qaz741wsd856/warden-worker) で **Watch** → **Custom** → **Releases** にチェックを入れます。新しいバージョンが公開されると通知されます。
2. **フォークを同期** — GitHub 上で自分のフォークを開き、**Sync fork** → **Update branch** をクリックします。上流の最新変更がフォークのデフォルトブランチに取り込まれます。
3. **自動デプロイ** — GitHub Actions で CI/CD を構築していれば、push-to-main ワークフローが新しいバージョンを自動的にビルドし Cloudflare Worker へデプロイします。手動作業は不要です。

> [!TIP]
> 最新の機能とセキュリティ修正を常に反映できるよう、上流で新しいリリースが公開されたらフォークを同期することを推奨します。

## コントリビューション

Issue や PR は歓迎します。提出前に `cargo fmt` と `cargo clippy --target wasm32-unknown-unknown --no-deps` を実行してください。

## ライセンス

このプロジェクトは MIT ライセンスの下で提供されています。詳細は `LICENSE` ファイルを参照してください。
