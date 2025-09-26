# Windows Makefile
TARGET = build/win64/twsearch.exe
CXX = g++
CXXFLAGS = -O3 -std=c++20 -I./src/cpp/vendor/cityhash/src -static -Wall -Wextra -Warray-bounds -pedantic -DUSE_PTHREADS -DUSE_PPQSORT
LDFLAGS = -static -static-libstdc++ -static-libgcc -lpthread

OBJ_DIR = build/cpp
BIN_DIR = build/win64
SRC_DIR = src/cpp

CPP_FILES = $(wildcard $(SRC_DIR)/*.cpp)
CC_FILES = $(SRC_DIR)/vendor/cityhash/src/city.cc

OBJS = $(patsubst $(SRC_DIR)/%.cpp,$(OBJ_DIR)/%.o,$(CPP_FILES))
OBJS += $(OBJ_DIR)/vendor/cityhash/city.o

all: $(OBJ_DIR) $(BIN_DIR) $(TARGET)

$(OBJ_DIR):
	if not exist "$(OBJ_DIR)" mkdir "$(OBJ_DIR)"
	if not exist "$(OBJ_DIR)/vendor/cityhash" mkdir "$(OBJ_DIR)/vendor/cityhash"

$(BIN_DIR):
	if not exist "$(BIN_DIR)" mkdir "$(BIN_DIR)"

$(OBJ_DIR)/%.o: $(SRC_DIR)/%.cpp | $(OBJ_DIR)
	$(CXX) $(CXXFLAGS) -c $< -o $@

$(OBJ_DIR)/vendor/cityhash/city.o: $(SRC_DIR)/vendor/cityhash/src/city.cc | $(OBJ_DIR)
	$(CXX) $(CXXFLAGS) -c $< -o $@

$(TARGET): $(OBJS) | $(BIN_DIR)
	$(CXX) $(CXXFLAGS) -o $@ $(OBJS) $(LDFLAGS)
	@echo Build completed: $(TARGET)

clean:
	-rmdir /s /q build 2>nul
	@echo Clean completed

test:
	$(TARGET) --help

.PHONY: all clean test