CC ?= gcc
LUAJIT ?= /usr/local/luajit
CFLAGS = -Wall -O2 -g -I$(LUAJIT)/include/luajit-2.1 -I.
LDFLAGS = -L$(LUAJIT)/lib -lluajit-5.1 -lm -ldl -lpthread

SRCS = lua_engine.c gui_tick.c
OBJS = $(SRCS:.c=.o)

.PHONY: all clean gui

all: libmy_design_agent.a my_design

libmy_design_agent.a: $(OBJS)
	ar rcs $@ $(OBJS)

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

my_design: main.c libmy_design_agent.a
	$(CC) $(CFLAGS) main.c gui_tick.o -o $@ -L. -lmy_design_agent $(LDFLAGS) -Wl,--export-dynamic

gui:
	cd gui_gpui && cargo build --release
	cp gui_gpui/target/release/libmy_design_gui.so .

clean:
	rm -f $(OBJS) libmy_design_agent.a libmy_design_gui.so my_design
	cd gui_gpui && cargo clean 2>/dev/null || true
