BASE_DIR=$(pwd)

for dir in ./canon_host/examples/*
do
		cd $dir
		./make.sh || exit
		cd $BASE_DIR		
done

cargo test "$@"
