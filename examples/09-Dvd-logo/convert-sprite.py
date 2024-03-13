import cv2
import numpy as np

m = cv2.imread("dvd.png")

h, w, bpp = np.shape(m)


def convert_to_words(data: list[list[int]]):
    final_words = []
    for row in data:
        words = []
        words.append(0)
        pixel = 0

        for col in row:
            if col == 1:
                words[len(words) - 1] |= 1 << pixel

            pixel += 1
            if pixel >= 16:
                # push the word & reset
                words.append(0)
                pixel = 0
        final_words.append(words)

    return final_words


def to_i16(val):
    result = np.array(val).astype(np.int16)
    as_string = "{}".format(result)
    if as_string == "-32768":
        return "32767+1"
    return as_string


def convert_to_jack(words: list[list[int]], function_name):
    print("    function void {}(int location) {{".format(function_name))
    print("       var int memAddress;")
    print("       let memAddress = 16384+location;")
    for i in range(len(words)):
        row = words[i]
        for j in range(len(row)):
            print(
                "       do Memory.poke(memAddress+{}, {});".format(
                    i * 32 + j, to_i16(row[j])
                )
            )
    print("       return;")
    print("    }")


def convert_colors(data: list[list[list[int]]]):
    new_data = []
    for row in data:
        new_row = []
        for col in row:
            if col[1] == 0:
                new_row.append(1)
            else:
                new_row.append(0)
        new_data.append(new_row)
    return new_data


def punt_row_data(data: list[list[int]], num_to_insert):
    for row in data:
        for _ in range(num_to_insert):
            row.insert(0, 0)


data = convert_colors(m)
punt_row_data(data, 0)
words = convert_to_words(data)
convert_to_jack(words, "drawFrame0")
