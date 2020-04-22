


src/%.rs: rocks/%.h
	bindgen --no-derive-debug \
		--no-layout-tests \
		--size_t-is-usize \
		--whitelist-function 'rocks.*?' \
		--whitelist-function 'cxx_.*?' \
		--whitelist-function 'free.*?' \
		--whitelist-type 'rocks.*?' \
		--whitelist-type 'cxx_.*?' \
		$^ -o $@

# --whitelist-type 'rust_.*?' \
# $^ -o $@

#all: src/options.rs \
#	src/rate_limiter.rs \
#	src/db.rs \
#	src/status.rs

all: src/c.rs


format:
	find rocks/ -iname '*.h' -o -iname '*.cc' -o -iname '*.hpp' | xargs clang-format -style=file -i
