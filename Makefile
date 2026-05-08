
run:
	cargo run
	
clean:
	cargo clean
	rm -Rf *.pkg.tar.zst git-flow-tui pkg dist

release: clean
	python resources/scripts/release.py