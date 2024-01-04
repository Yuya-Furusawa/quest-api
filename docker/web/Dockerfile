# ベースイメージを指定
FROM rust:1.73

# 作業ディレクトリを設定
WORKDIR /usr/src/app

# ソースコードとCargo.tomlをコピー
COPY . .

# sqlx-cliツールをインストール
RUN cargo install sqlx-cli cargo-watch

# 依存関係をビルドし、キャッシュを利用する
RUN cargo build --release

# ポートを公開
EXPOSE 3000

# 環境変数を設定
ENV DATABASE_URL=$DATABASE_URL

# 実行コマンドを指定
CMD ["cargo", "run", "--release"]