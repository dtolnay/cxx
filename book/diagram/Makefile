all:

bin/asciitosvg:
	rm -rf build
	mkdir -p bin build/src/github.com/asciitosvg/asciitosvg
	cd build/src/github.com/asciitosvg/asciitosvg && \
		git init && \
	    git remote add asciitosvg https://github.com/asciitosvg/asciitosvg && \
		git fetch asciitosvg refs/pull/23/head && \
		git checkout --quiet 29f2ed76b60231da99ac1d4bca26e8ff93125a2e
	GOPATH=${CURDIR}/build go build -o $@ ./build/src/github.com/asciitosvg/asciitosvg/cmd/a2s

%.svg: %.ascii bin/asciitosvg
	bin/asciitosvg -i $< -o $@ -b

clean:
	rm -f *.svg

.PHONY: all clean
