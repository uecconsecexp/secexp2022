package connector

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
)

func testMatrix(yobikou YobikouServer, chugaku ChugakuClient) func(t *testing.T) {
	return func(t *testing.T) {
		table := [][]float64{{1, 2, 3}, {4, 5, 6}, {7, 8, 9}}
		t.Run("ping matrix", testPingMatrix(yobikou, chugaku, table))
		t.Run("pong matrix", testPingMatrix(chugaku, yobikou, table))
	}
}

type receiveMatRes struct {
	res [][]float64
	err error
}

func testPingMatrix(m1 Messenger, m2 Messenger, matrix [][]float64) func(t *testing.T) {
	return func(t *testing.T) {
		fmt.Println("testPingMatrix")
		go m1.SendTable(matrix)

		c := make(chan receiveMatRes)
		go func() {
			res, err := m2.ReceiveTable()
			c <- receiveMatRes{res, err}
		}()

		// time.Sleep(SPAN)

		result := <-c
		if result.err != nil {
			t.Error(result.err)
		}
		assert.Equal(t, matrix, result.res)
		fmt.Println("testPingMatrix done")
	}
}
