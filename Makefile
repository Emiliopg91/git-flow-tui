
run:
	@cargo run
	
clean:
	@cargo clean
	@rm -Rf *.pkg.tar.zst git-flow-tui pkg dist tests/repo

release: clean
	@python resources/scripts/release.py

test:
	@RUST_BACKTRACE=1 cargo test -- --no-capture --test-threads=1