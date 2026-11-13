cargo build -r

cd apps/web
pnpm i
pnpm build
cd build
touch watchtower-webpage.tar.gz
tar --exclude="watchtower-webpage.tar.gz" -czf watchtower-webpage.tar.gz .

# cd back to the project root
cd ../../../

rm -rf build
mkdir build

cp target/release/watchtower-agent build/watchtower-agent
cp target/release/watchtower-server build/watchtower-server
mv apps/web/build/watchtower-webpage.tar.gz build/watchtower-webpage.tar.gz