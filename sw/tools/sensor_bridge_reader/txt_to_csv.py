import os

DATAFILE = "/Users/anthony/Downloads/first_car_data.txt"
OUTFILE = "/Users/anthony/Downloads/first_car_data_csv.txt"

def parse_datafile_to_blocks(datafile):
    blocks = []
    current_block = []

    for line in datafile:
        line = line.rstrip("\n")
        if line:
            current_block.append(line)
        elif current_block:
            blocks.append(current_block)
            current_block = []

    if current_block:
        blocks.append(current_block)

    return blocks

def parse_data(datafile, outfile):
    blocks = parse_datafile_to_blocks(datafile)
    print(blocks[0])

    # for line in lines():
    #     if line.startswith("CAN ID: 48"):


if __name__ == "__main__":
    with open(DATAFILE, "r") as datafile:
        with open(OUTFILE, "w") as outfile:
            parse_data(datafile, outfile);
