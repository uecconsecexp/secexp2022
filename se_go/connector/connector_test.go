package connector

import (
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

var SPAN = time.Duration(10) * time.Millisecond

func TestConnector(t *testing.T) {
	yc := make(chan YobikouServer)
	tc := make(chan ChugakuClient)

	go prepareServer(yc)
	time.Sleep(SPAN)
	go prepareClient(tc)

	yobikou := <-yc
	defer yobikou.Close()
	chugaku := <-tc
	defer chugaku.Close()

	for i := 0; i < 10; i++ {
		t.Run("ping", testPing(yobikou, chugaku))
		t.Run("pong", testPing(chugaku, yobikou))
		t.Run("pingWithNewLine", testPingWithNewLine(yobikou, chugaku))
		t.Run("pongWithNewLine", testPingWithNewLine(chugaku, yobikou))
		t.Run("matrix", testMatrix(yobikou, chugaku))
	}
}

func prepareServer(ch chan YobikouServer) {
	yobikou, err := NewYobikouServer()
	if err != nil {
		panic(err)
	}

	ch <- yobikou
}

func prepareClient(ch chan ChugakuClient) {
	chugaku, err := NewChugakuClient("0.0.0.0")
	if err != nil {
		panic(err)
	}

	ch <- chugaku
}

type Messenger interface {
	Send([]byte) error
	Receive() ([]byte, error)
	SendTable([][]float64) error
	ReceiveTable() ([][]float64, error)
}

type receiveRes struct {
	res []byte
	err error
}

func testPing(m1 Messenger, m2 Messenger) func(t *testing.T) {
	return func(t *testing.T) {
		fmt.Println("testPing")
		go m1.Send([]byte("ping"))

		c := make(chan receiveRes)
		go func() {
			result, err := m2.Receive()
			c <- receiveRes{result, err}
		}()

		r := <-c
		if r.err != nil {
			t.Fatal(r.err)
		}
		assert.Equal(t, "ping", string(r.res))
		fmt.Println("testPing done")
	}
}

func testPingWithNewLine(m1 Messenger, m2 Messenger) func(t *testing.T) {
	return func(t *testing.T) {
		fmt.Println("testWithNewLine")
		go m1.Send([]byte(`Hello,
		Ping!`))

		c := make(chan receiveRes)
		go func() {
			result, err := m2.Receive()
			c <- receiveRes{result, err}
		}()

		r := <-c
		if r.err != nil {
			t.Fatal(r.err)
		}
		assert.Equal(t, `Hello,
		Ping!`, string(r.res))
		fmt.Println("testWithNewLine done")
	}
}
