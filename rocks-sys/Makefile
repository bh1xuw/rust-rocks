


src/%.rs: rocks/%.h
	bindgen --no-derive-debug \
		--no-layout-tests \
		$^ -o $@
#		--whitelist-function 'rocks.*?' \
#		--whitelist-type 'rocks.*?' \
		$^ -o $@

#all: src/options.rs \
#	src/rate_limiter.rs \
#	src/db.rs \
#	src/status.rs

all: src/c.rs
