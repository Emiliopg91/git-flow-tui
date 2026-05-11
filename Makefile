
run:
	@cargo run
	
clean:
	@cargo clean
	@rm -Rf *.pkg.tar.zst git-flow-tui pkg dist tests/repo

release: clean
	@python resources/scripts/release.py

test:
	@cargo test -- --no-capture --test-threads=1