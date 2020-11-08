all:

bin/asciitosvg:
	rm -rf build
	mkdir -p bin build/src/github.com/asciitosvg
	git clone https://github.com/dtolnay/asciitosvg build/src/github.com/asciitosvg/asciitosvg
	GOPATH=${CURDIR}/build go build -o $@ ./build/src/github.com/asciitosvg/asciitosvg/cmd/a2s

%.svg: %.ascii bin/asciitosvg
	bin/asciitosvg -i $< -o $@ -b

clean:
	rm -f *.svg

.PHONY: all clean
