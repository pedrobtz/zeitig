
// clang-format sorts includes unless SortIncludes: Never. However, the ordering
// does matter here. So, we need to disable clang-format for safety.

// clang-format off
#include <stdint.h>
#include <Rinternals.h>
#include <R_ext/Parse.h>
// clang-format on

#include "rust/api.h"

static uintptr_t TAGGED_POINTER_MASK = (uintptr_t)1;

SEXP handle_result(SEXP res_) {
    uintptr_t res = (uintptr_t)res_;

    // An error is indicated by tag.
    if ((res & TAGGED_POINTER_MASK) == 1) {
        // Remove tag
        SEXP res_aligned = (SEXP)(res & ~TAGGED_POINTER_MASK);

        // Currently, there are two types of error cases:
        //
        //   1. Error from Rust code
        //   2. Error from R's C API, which is caught by R_UnwindProtect()
        //
        if (TYPEOF(res_aligned) == CHARSXP) {
            // In case 1, the result is an error message that can be passed to
            // Rf_errorcall() directly.
            Rf_errorcall(R_NilValue, "%s", CHAR(res_aligned));
        } else {
            // In case 2, the result is the token to restart the
            // cleanup process on R's side.
            R_ContinueUnwind(res_aligned);
        }
    }

    return (SEXP)res;
}

SEXP savvy_rs_available_time_zones__impl(void) {
    SEXP res = savvy_rs_available_time_zones__ffi();
    return handle_result(res);
}

SEXP savvy_rs_duration_add__impl(SEXP c_arg__x, SEXP c_arg__y) {
    SEXP res = savvy_rs_duration_add__ffi(c_arg__x, c_arg__y);
    return handle_result(res);
}

SEXP savvy_rs_duration_compare__impl(SEXP c_arg__x, SEXP c_arg__y, SEXP c_arg__relative) {
    SEXP res = savvy_rs_duration_compare__ffi(c_arg__x, c_arg__y, c_arg__relative);
    return handle_result(res);
}

SEXP savvy_rs_duration_format__impl(SEXP c_arg__x) {
    SEXP res = savvy_rs_duration_format__ffi(c_arg__x);
    return handle_result(res);
}

SEXP savvy_rs_duration_parse__impl(SEXP c_arg__x) {
    SEXP res = savvy_rs_duration_parse__ffi(c_arg__x);
    return handle_result(res);
}

SEXP savvy_rs_duration_round__impl(SEXP c_arg__x, SEXP c_arg__largest, SEXP c_arg__smallest, SEXP c_arg__increment, SEXP c_arg__mode, SEXP c_arg__relative) {
    SEXP res = savvy_rs_duration_round__ffi(c_arg__x, c_arg__largest, c_arg__smallest, c_arg__increment, c_arg__mode, c_arg__relative);
    return handle_result(res);
}

SEXP savvy_rs_duration_sort_key__impl(SEXP c_arg__x) {
    SEXP res = savvy_rs_duration_sort_key__ffi(c_arg__x);
    return handle_result(res);
}

SEXP savvy_rs_duration_total__impl(SEXP c_arg__x, SEXP c_arg__unit, SEXP c_arg__relative) {
    SEXP res = savvy_rs_duration_total__ffi(c_arg__x, c_arg__unit, c_arg__relative);
    return handle_result(res);
}

SEXP savvy_rs_duration_validate__impl(SEXP c_arg__x) {
    SEXP res = savvy_rs_duration_validate__ffi(c_arg__x);
    return handle_result(res);
}

SEXP savvy_rs_plain_date_add__impl(SEXP c_arg__year, SEXP c_arg__month, SEXP c_arg__day, SEXP c_arg__duration, SEXP c_arg__reject) {
    SEXP res = savvy_rs_plain_date_add__ffi(c_arg__year, c_arg__month, c_arg__day, c_arg__duration, c_arg__reject);
    return handle_result(res);
}

SEXP savvy_rs_plain_date_diff__impl(SEXP c_arg__x, SEXP c_arg__y, SEXP c_arg__largest, SEXP c_arg__smallest, SEXP c_arg__increment, SEXP c_arg__mode, SEXP c_arg__since) {
    SEXP res = savvy_rs_plain_date_diff__ffi(c_arg__x, c_arg__y, c_arg__largest, c_arg__smallest, c_arg__increment, c_arg__mode, c_arg__since);
    return handle_result(res);
}

SEXP savvy_rs_plain_date_field__impl(SEXP c_arg__year, SEXP c_arg__month, SEXP c_arg__day, SEXP c_arg__field) {
    SEXP res = savvy_rs_plain_date_field__ffi(c_arg__year, c_arg__month, c_arg__day, c_arg__field);
    return handle_result(res);
}

SEXP savvy_rs_plain_date_format__impl(SEXP c_arg__year, SEXP c_arg__month, SEXP c_arg__day) {
    SEXP res = savvy_rs_plain_date_format__ffi(c_arg__year, c_arg__month, c_arg__day);
    return handle_result(res);
}

SEXP savvy_rs_plain_date_from_epoch_days__impl(SEXP c_arg__days) {
    SEXP res = savvy_rs_plain_date_from_epoch_days__ffi(c_arg__days);
    return handle_result(res);
}

SEXP savvy_rs_plain_date_from_parts__impl(SEXP c_arg__year, SEXP c_arg__month, SEXP c_arg__day, SEXP c_arg__reject) {
    SEXP res = savvy_rs_plain_date_from_parts__ffi(c_arg__year, c_arg__month, c_arg__day, c_arg__reject);
    return handle_result(res);
}

SEXP savvy_rs_plain_date_parse__impl(SEXP c_arg__x) {
    SEXP res = savvy_rs_plain_date_parse__ffi(c_arg__x);
    return handle_result(res);
}

SEXP savvy_rs_plain_date_time_add__impl(SEXP c_arg__x, SEXP c_arg__duration, SEXP c_arg__reject) {
    SEXP res = savvy_rs_plain_date_time_add__ffi(c_arg__x, c_arg__duration, c_arg__reject);
    return handle_result(res);
}

SEXP savvy_rs_plain_date_time_diff__impl(SEXP c_arg__x, SEXP c_arg__y, SEXP c_arg__largest, SEXP c_arg__smallest, SEXP c_arg__increment, SEXP c_arg__mode, SEXP c_arg__since) {
    SEXP res = savvy_rs_plain_date_time_diff__ffi(c_arg__x, c_arg__y, c_arg__largest, c_arg__smallest, c_arg__increment, c_arg__mode, c_arg__since);
    return handle_result(res);
}

SEXP savvy_rs_plain_date_time_format__impl(SEXP c_arg__year, SEXP c_arg__month, SEXP c_arg__day, SEXP c_arg__second_of_day, SEXP c_arg__nanos) {
    SEXP res = savvy_rs_plain_date_time_format__ffi(c_arg__year, c_arg__month, c_arg__day, c_arg__second_of_day, c_arg__nanos);
    return handle_result(res);
}

SEXP savvy_rs_plain_date_time_parse__impl(SEXP c_arg__x) {
    SEXP res = savvy_rs_plain_date_time_parse__ffi(c_arg__x);
    return handle_result(res);
}

SEXP savvy_rs_plain_date_time_round__impl(SEXP c_arg__x, SEXP c_arg__smallest, SEXP c_arg__increment, SEXP c_arg__mode) {
    SEXP res = savvy_rs_plain_date_time_round__ffi(c_arg__x, c_arg__smallest, c_arg__increment, c_arg__mode);
    return handle_result(res);
}

SEXP savvy_rs_plain_date_to_epoch_days__impl(SEXP c_arg__year, SEXP c_arg__month, SEXP c_arg__day) {
    SEXP res = savvy_rs_plain_date_to_epoch_days__ffi(c_arg__year, c_arg__month, c_arg__day);
    return handle_result(res);
}

SEXP savvy_rs_plain_time_add__impl(SEXP c_arg__second_of_day, SEXP c_arg__nanos, SEXP c_arg__duration) {
    SEXP res = savvy_rs_plain_time_add__ffi(c_arg__second_of_day, c_arg__nanos, c_arg__duration);
    return handle_result(res);
}

SEXP savvy_rs_plain_time_diff__impl(SEXP c_arg__x, SEXP c_arg__y, SEXP c_arg__largest, SEXP c_arg__smallest, SEXP c_arg__increment, SEXP c_arg__mode, SEXP c_arg__since) {
    SEXP res = savvy_rs_plain_time_diff__ffi(c_arg__x, c_arg__y, c_arg__largest, c_arg__smallest, c_arg__increment, c_arg__mode, c_arg__since);
    return handle_result(res);
}

SEXP savvy_rs_plain_time_format__impl(SEXP c_arg__second_of_day, SEXP c_arg__nanos) {
    SEXP res = savvy_rs_plain_time_format__ffi(c_arg__second_of_day, c_arg__nanos);
    return handle_result(res);
}

SEXP savvy_rs_plain_time_from_parts__impl(SEXP c_arg__hour, SEXP c_arg__minute, SEXP c_arg__second, SEXP c_arg__millisecond, SEXP c_arg__microsecond, SEXP c_arg__nanosecond, SEXP c_arg__reject) {
    SEXP res = savvy_rs_plain_time_from_parts__ffi(c_arg__hour, c_arg__minute, c_arg__second, c_arg__millisecond, c_arg__microsecond, c_arg__nanosecond, c_arg__reject);
    return handle_result(res);
}

SEXP savvy_rs_plain_time_parse__impl(SEXP c_arg__x) {
    SEXP res = savvy_rs_plain_time_parse__ffi(c_arg__x);
    return handle_result(res);
}

SEXP savvy_rs_plain_time_round__impl(SEXP c_arg__second_of_day, SEXP c_arg__nanos, SEXP c_arg__smallest, SEXP c_arg__increment, SEXP c_arg__mode) {
    SEXP res = savvy_rs_plain_time_round__ffi(c_arg__second_of_day, c_arg__nanos, c_arg__smallest, c_arg__increment, c_arg__mode);
    return handle_result(res);
}


static const R_CallMethodDef CallEntries[] = {
    {"savvy_rs_available_time_zones__impl", (DL_FUNC) &savvy_rs_available_time_zones__impl, 0},
    {"savvy_rs_duration_add__impl", (DL_FUNC) &savvy_rs_duration_add__impl, 2},
    {"savvy_rs_duration_compare__impl", (DL_FUNC) &savvy_rs_duration_compare__impl, 3},
    {"savvy_rs_duration_format__impl", (DL_FUNC) &savvy_rs_duration_format__impl, 1},
    {"savvy_rs_duration_parse__impl", (DL_FUNC) &savvy_rs_duration_parse__impl, 1},
    {"savvy_rs_duration_round__impl", (DL_FUNC) &savvy_rs_duration_round__impl, 6},
    {"savvy_rs_duration_sort_key__impl", (DL_FUNC) &savvy_rs_duration_sort_key__impl, 1},
    {"savvy_rs_duration_total__impl", (DL_FUNC) &savvy_rs_duration_total__impl, 3},
    {"savvy_rs_duration_validate__impl", (DL_FUNC) &savvy_rs_duration_validate__impl, 1},
    {"savvy_rs_plain_date_add__impl", (DL_FUNC) &savvy_rs_plain_date_add__impl, 5},
    {"savvy_rs_plain_date_diff__impl", (DL_FUNC) &savvy_rs_plain_date_diff__impl, 7},
    {"savvy_rs_plain_date_field__impl", (DL_FUNC) &savvy_rs_plain_date_field__impl, 4},
    {"savvy_rs_plain_date_format__impl", (DL_FUNC) &savvy_rs_plain_date_format__impl, 3},
    {"savvy_rs_plain_date_from_epoch_days__impl", (DL_FUNC) &savvy_rs_plain_date_from_epoch_days__impl, 1},
    {"savvy_rs_plain_date_from_parts__impl", (DL_FUNC) &savvy_rs_plain_date_from_parts__impl, 4},
    {"savvy_rs_plain_date_parse__impl", (DL_FUNC) &savvy_rs_plain_date_parse__impl, 1},
    {"savvy_rs_plain_date_time_add__impl", (DL_FUNC) &savvy_rs_plain_date_time_add__impl, 3},
    {"savvy_rs_plain_date_time_diff__impl", (DL_FUNC) &savvy_rs_plain_date_time_diff__impl, 7},
    {"savvy_rs_plain_date_time_format__impl", (DL_FUNC) &savvy_rs_plain_date_time_format__impl, 5},
    {"savvy_rs_plain_date_time_parse__impl", (DL_FUNC) &savvy_rs_plain_date_time_parse__impl, 1},
    {"savvy_rs_plain_date_time_round__impl", (DL_FUNC) &savvy_rs_plain_date_time_round__impl, 4},
    {"savvy_rs_plain_date_to_epoch_days__impl", (DL_FUNC) &savvy_rs_plain_date_to_epoch_days__impl, 3},
    {"savvy_rs_plain_time_add__impl", (DL_FUNC) &savvy_rs_plain_time_add__impl, 3},
    {"savvy_rs_plain_time_diff__impl", (DL_FUNC) &savvy_rs_plain_time_diff__impl, 7},
    {"savvy_rs_plain_time_format__impl", (DL_FUNC) &savvy_rs_plain_time_format__impl, 2},
    {"savvy_rs_plain_time_from_parts__impl", (DL_FUNC) &savvy_rs_plain_time_from_parts__impl, 7},
    {"savvy_rs_plain_time_parse__impl", (DL_FUNC) &savvy_rs_plain_time_parse__impl, 1},
    {"savvy_rs_plain_time_round__impl", (DL_FUNC) &savvy_rs_plain_time_round__impl, 5},
    {NULL, NULL, 0}
};

void R_init_zudate(DllInfo *dll) {
    R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);
    R_useDynamicSymbols(dll, FALSE);

    // Functions for initialization, if any.

}
