package main

import (
	"errors"
	"fmt"
	"log/slog"
	"os"

	cerrors "github.com/cockroachdb/errors"
	ferrors "github.com/friendsofgo/errors"
	gerrors "github.com/go-errors/errors"
)

func layer1_1() error {
	return cerrors.New("original error 1")
}

func layer1_2() error {
	return cerrors.New("original error 2")
}

func layer2() error {
	err1 := layer1_1()
	err2 := layer1_2()
	return cerrors.Join(err1, err2)
}

func layer3() error {
	return layer2()
}

func main() {
	err := layer3()
	logger := slog.New(slog.NewJSONHandler(os.Stdout, nil))

	if err != nil {
		logger.Error("cockroach errors message", "err", err, "v", fmt.Sprintf("%+v", err), "GetAllSafeDetails", cerrors.GetAllSafeDetails(err), "GetReportableStackTrace", cerrors.GetReportableStackTrace(err))
		if err, ok := err.(stackTracer); ok {
			logger.Error("friendsofgo error message", "err", err, "StackTrace", err.StackTrace())
		}
		var goErr *gerrors.Error
		if ok := errors.As(err, &goErr); ok {
			logger.Error("go-errors errors message", "err", err, "StackFrames", goErr.StackFrames())
		}
	}
}

type stackTracer interface {
	StackTrace() ferrors.StackTrace
}
