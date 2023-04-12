# ベースイメージを指定
FROM rust:1.68

# 作業ディレクトリを設定
WORKDIR /usr/src/app

# ソースコードとCargo.tomlをコピー
COPY . .

# 依存関係をビルドし、キャッシュを利用する
RUN cargo build --release

# ポートを公開
EXPOSE 3000

# 実行コマンドを指定
CMD ["cargo", "run", "--release"]