#include <X11/Xatom.h>
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <X11/extensions/XTest.h>

#include <errno.h>
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

typedef struct {
  Window window;
  unsigned long area;
} WindowMatch;

static volatile int x_error_occurred = 0;

static int ignore_x_error(Display *display, XErrorEvent *error) {
  (void)display;
  (void)error;
  x_error_occurred = 1;
  return 0;
}

static void die(const char *message) {
  fprintf(stderr, "sonar-x11-input: %s\n", message);
  exit(2);
}

static long parse_long(const char *value, const char *label) {
  char *end = NULL;
  errno = 0;
  long parsed = strtol(value, &end, 0);
  if (errno != 0 || end == value || *end != '\0') {
    fprintf(stderr, "sonar-x11-input: invalid %s: %s\n", label, value);
    exit(2);
  }
  return parsed;
}

static Window parse_window(const char *value) {
  long parsed = parse_long(value, "window id");
  if (parsed <= 0) {
    die("window id must be positive");
  }
  return (Window)parsed;
}

static long monotonic_millis(void) {
  struct timespec value;
  if (clock_gettime(CLOCK_MONOTONIC, &value) != 0) {
    die("clock_gettime failed");
  }
  return value.tv_sec * 1000L + value.tv_nsec / 1000000L;
}

static char *window_name(Display *display, Window window) {
  Atom utf8 = XInternAtom(display, "UTF8_STRING", False);
  Atom net_wm_name = XInternAtom(display, "_NET_WM_NAME", False);
  Atom actual_type;
  int actual_format;
  unsigned long item_count;
  unsigned long bytes_after;
  unsigned char *property = NULL;

  if (XGetWindowProperty(display, window, net_wm_name, 0, 4096, False, utf8,
                         &actual_type, &actual_format, &item_count,
                         &bytes_after, &property) == Success &&
      property != NULL && actual_format == 8) {
    char *result = strdup((const char *)property);
    XFree(property);
    return result;
  }
  if (property != NULL) {
    XFree(property);
  }

  char *legacy_name = NULL;
  if (XFetchName(display, window, &legacy_name) != 0 && legacy_name != NULL) {
    char *result = strdup(legacy_name);
    XFree(legacy_name);
    return result;
  }
  return NULL;
}

static bool is_viewable_large_window(Display *display, Window window,
                                     XWindowAttributes *attributes) {
  if (XGetWindowAttributes(display, window, attributes) == 0) {
    return false;
  }
  return attributes->map_state == IsViewable && attributes->width >= 100 &&
         attributes->height >= 100;
}

static void find_named_recursive(Display *display, Window parent,
                                 const char *needle, WindowMatch *best) {
  Window root;
  Window parent_return;
  Window *children = NULL;
  unsigned int child_count = 0;

  if (XQueryTree(display, parent, &root, &parent_return, &children,
                 &child_count) == 0) {
    return;
  }

  for (unsigned int index = 0; index < child_count; index++) {
    Window child = children[index];
    XWindowAttributes attributes;
    if (is_viewable_large_window(display, child, &attributes)) {
      char *name = window_name(display, child);
      if (name != NULL && strstr(name, needle) != NULL) {
        unsigned long area =
            (unsigned long)attributes.width * (unsigned long)attributes.height;
        if (area > best->area) {
          best->window = child;
          best->area = area;
        }
      }
      free(name);
    }
    find_named_recursive(display, child, needle, best);
  }

  if (children != NULL) {
    XFree(children);
  }
}

static Window find_named_window(Display *display, const char *needle) {
  WindowMatch best = {0, 0};
  find_named_recursive(display, DefaultRootWindow(display), needle, &best);
  return best.window;
}

static bool same_window_class(Display *display, Window left, Window right) {
  XClassHint left_hint = {0};
  XClassHint right_hint = {0};
  bool same = false;

  if (XGetClassHint(display, left, &left_hint) != 0 &&
      XGetClassHint(display, right, &right_hint) != 0) {
    const char *left_name = left_hint.res_name == NULL ? "" : left_hint.res_name;
    const char *right_name =
        right_hint.res_name == NULL ? "" : right_hint.res_name;
    const char *left_class =
        left_hint.res_class == NULL ? "" : left_hint.res_class;
    const char *right_class =
        right_hint.res_class == NULL ? "" : right_hint.res_class;
    same = strcmp(left_name, right_name) == 0 &&
           strcmp(left_class, right_class) == 0;
  }

  if (left_hint.res_name != NULL) {
    XFree(left_hint.res_name);
  }
  if (left_hint.res_class != NULL) {
    XFree(left_hint.res_class);
  }
  if (right_hint.res_name != NULL) {
    XFree(right_hint.res_name);
  }
  if (right_hint.res_class != NULL) {
    XFree(right_hint.res_class);
  }
  return same;
}

static void find_dialog_recursive(Display *display, Window parent, Window main,
                                  WindowMatch *best) {
  Window root;
  Window parent_return;
  Window *children = NULL;
  unsigned int child_count = 0;

  if (XQueryTree(display, parent, &root, &parent_return, &children,
                 &child_count) == 0) {
    return;
  }

  for (unsigned int index = 0; index < child_count; index++) {
    Window child = children[index];
    XWindowAttributes attributes;
    if (child != main && is_viewable_large_window(display, child, &attributes)) {
      Window transient_for = None;
      bool is_transient =
          XGetTransientForHint(display, child, &transient_for) != 0 &&
          transient_for == main;
      if (is_transient || same_window_class(display, child, main)) {
        char *name = window_name(display, child);
        if (name != NULL) {
          unsigned long area = (unsigned long)attributes.width *
                               (unsigned long)attributes.height;
          if (area > best->area) {
            best->window = child;
            best->area = area;
          }
        }
        free(name);
      }
    }
    find_dialog_recursive(display, child, main, best);
  }

  if (children != NULL) {
    XFree(children);
  }
}

static Window find_dialog(Display *display, Window main) {
  WindowMatch best = {0, 0};
  find_dialog_recursive(display, DefaultRootWindow(display), main, &best);
  return best.window;
}

static Window wait_for_named_window(Display *display, const char *needle,
                                    long timeout_ms) {
  long deadline = monotonic_millis() + timeout_ms;
  do {
    Window found = find_named_window(display, needle);
    if (found != None) {
      return found;
    }
    usleep(100000);
  } while (monotonic_millis() < deadline);
  return None;
}

static Window wait_for_dialog(Display *display, Window main, long timeout_ms) {
  long deadline = monotonic_millis() + timeout_ms;
  do {
    Window found = find_dialog(display, main);
    if (found != None) {
      return found;
    }
    usleep(100000);
  } while (monotonic_millis() < deadline);
  return None;
}

static bool window_exists(Display *display, Window window) {
  XWindowAttributes attributes;
  x_error_occurred = 0;
  int success = XGetWindowAttributes(display, window, &attributes);
  XSync(display, False);
  return success != 0 && x_error_occurred == 0 &&
         attributes.map_state == IsViewable;
}

static bool window_manager_is_running(Display *display) {
  Atom wm_check = XInternAtom(display, "_NET_SUPPORTING_WM_CHECK", False);
  Atom actual_type;
  int actual_format;
  unsigned long item_count;
  unsigned long bytes_after;
  unsigned char *property = NULL;
  int result = XGetWindowProperty(
      display, DefaultRootWindow(display), wm_check, 0, 1, False, XA_WINDOW,
      &actual_type, &actual_format, &item_count, &bytes_after, &property);
  bool running = result == Success && property != NULL && item_count == 1;
  if (property != NULL) {
    XFree(property);
  }
  return running;
}

static void focus_window(Display *display, Window window) {
  Window root = DefaultRootWindow(display);
  if (window_manager_is_running(display)) {
    Atom active_window = XInternAtom(display, "_NET_ACTIVE_WINDOW", False);
    XEvent event;
    memset(&event, 0, sizeof(event));
    event.xclient.type = ClientMessage;
    event.xclient.window = window;
    event.xclient.message_type = active_window;
    event.xclient.format = 32;
    event.xclient.data.l[0] = 2;
    event.xclient.data.l[1] = CurrentTime;
    XSendEvent(display, root, False,
               SubstructureRedirectMask | SubstructureNotifyMask, &event);
  } else {
    XRaiseWindow(display, window);
    XSetInputFocus(display, window, RevertToParent, CurrentTime);
  }
  XSync(display, False);
  usleep(200000);
}

static void fake_key(Display *display, KeySym symbol, Bool pressed) {
  KeyCode code = XKeysymToKeycode(display, symbol);
  if (code == 0) {
    fprintf(stderr, "sonar-x11-input: missing keycode for keysym %lu\n",
            symbol);
    exit(2);
  }
  XTestFakeKeyEvent(display, code, pressed, CurrentTime);
}

static void send_key(Display *display, Window window, const char *key_name,
                     bool control, bool shift) {
  KeySym symbol = XStringToKeysym(key_name);
  if (symbol == NoSymbol) {
    fprintf(stderr, "sonar-x11-input: unknown keysym: %s\n", key_name);
    exit(2);
  }

  // Le script appelle focus-id lors d'un changement de fenêtre. Ne pas
  // réassigner ici le focus X au top-level : cela ferait perdre au dialogue
  // GTK le focus interne posé par Ctrl+L sur son champ de chemin.
  (void)window;
  if (control) {
    fake_key(display, XK_Control_L, True);
  }
  if (shift) {
    fake_key(display, XK_Shift_L, True);
  }
  fake_key(display, symbol, True);
  fake_key(display, symbol, False);
  if (shift) {
    fake_key(display, XK_Shift_L, False);
  }
  if (control) {
    fake_key(display, XK_Control_L, False);
  }
  XSync(display, False);
  usleep(100000);
}

static void click_window(Display *display, Window window, int x, int y) {
  XWindowAttributes attributes;
  if (XGetWindowAttributes(display, window, &attributes) == 0) {
    die("cannot read target window geometry");
  }
  if (x < 0 || y < 0 || x >= attributes.width || y >= attributes.height) {
    die("click coordinates are outside the target window");
  }

  Window child;
  int root_x;
  int root_y;
  if (XTranslateCoordinates(display, window, DefaultRootWindow(display), x, y,
                            &root_x, &root_y, &child) == 0) {
    die("cannot translate click coordinates");
  }

  focus_window(display, window);
  XTestFakeMotionEvent(display, DefaultScreen(display), root_x, root_y,
                       CurrentTime);
  XTestFakeButtonEvent(display, 1, True, CurrentTime);
  XTestFakeButtonEvent(display, 1, False, CurrentTime);
  XSync(display, False);
  usleep(100000);
}

static void print_geometry(Display *display, Window window) {
  XWindowAttributes attributes;
  Window child;
  int root_x;
  int root_y;
  if (XGetWindowAttributes(display, window, &attributes) == 0 ||
      XTranslateCoordinates(display, window, DefaultRootWindow(display), 0, 0,
                            &root_x, &root_y, &child) == 0) {
    die("cannot read target window geometry");
  }
  printf("%d %d %d %d\n", attributes.width, attributes.height, root_x,
         root_y);
}

static void print_usage(const char *program) {
  fprintf(stderr,
          "usage:\n"
          "  %s wait-window TITLE TIMEOUT_MS\n"
          "  %s wait-dialog MAIN_WINDOW_ID TIMEOUT_MS\n"
          "  %s wait-gone WINDOW_ID TIMEOUT_MS\n"
          "  %s geometry-id WINDOW_ID\n"
          "  %s focus-id WINDOW_ID\n"
          "  %s click-id WINDOW_ID X Y\n"
          "  %s key-id WINDOW_ID KEYSYM\n"
          "  %s ctrl-id WINDOW_ID KEYSYM\n"
          "  %s ctrl-shift-id WINDOW_ID KEYSYM\n",
          program, program, program, program, program, program, program,
          program, program);
}

int main(int argc, char **argv) {
  Display *display = XOpenDisplay(NULL);
  if (display == NULL) {
    die("cannot open X display");
  }
  XSetErrorHandler(ignore_x_error);

  int status = 0;
  if (argc == 4 && strcmp(argv[1], "wait-window") == 0) {
    long timeout_ms = parse_long(argv[3], "timeout");
    Window found = wait_for_named_window(display, argv[2], timeout_ms);
    if (found == None) {
      fprintf(stderr, "sonar-x11-input: window not found: %s\n", argv[2]);
      status = 1;
    } else {
      printf("0x%lx\n", found);
    }
  } else if (argc == 4 && strcmp(argv[1], "wait-dialog") == 0) {
    Window main = parse_window(argv[2]);
    long timeout_ms = parse_long(argv[3], "timeout");
    Window found = wait_for_dialog(display, main, timeout_ms);
    if (found == None) {
      fputs("sonar-x11-input: native dialog not found\n", stderr);
      status = 1;
    } else {
      printf("0x%lx\n", found);
    }
  } else if (argc == 4 && strcmp(argv[1], "wait-gone") == 0) {
    Window window = parse_window(argv[2]);
    long timeout_ms = parse_long(argv[3], "timeout");
    long deadline = monotonic_millis() + timeout_ms;
    while (window_exists(display, window) && monotonic_millis() < deadline) {
      usleep(100000);
    }
    if (window_exists(display, window)) {
      fputs("sonar-x11-input: window did not close\n", stderr);
      status = 1;
    }
  } else if (argc == 3 && strcmp(argv[1], "geometry-id") == 0) {
    print_geometry(display, parse_window(argv[2]));
  } else if (argc == 3 && strcmp(argv[1], "focus-id") == 0) {
    focus_window(display, parse_window(argv[2]));
  } else if (argc == 5 && strcmp(argv[1], "click-id") == 0) {
    click_window(display, parse_window(argv[2]),
                 (int)parse_long(argv[3], "x coordinate"),
                 (int)parse_long(argv[4], "y coordinate"));
  } else if (argc == 4 && strcmp(argv[1], "key-id") == 0) {
    send_key(display, parse_window(argv[2]), argv[3], false, false);
  } else if (argc == 4 && strcmp(argv[1], "ctrl-id") == 0) {
    send_key(display, parse_window(argv[2]), argv[3], true, false);
  } else if (argc == 4 && strcmp(argv[1], "ctrl-shift-id") == 0) {
    send_key(display, parse_window(argv[2]), argv[3], true, true);
  } else {
    print_usage(argv[0]);
    status = 2;
  }

  XCloseDisplay(display);
  return status;
}
