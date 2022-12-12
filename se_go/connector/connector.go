package connector

import (
	"bufio"
	"bytes"
	"fmt"
	"net"
	"time"
)

const PORT = ":10000"
const PROTOCOL = "tcp"
const DEADLINE = time.Duration(10) * time.Second

type Communicator interface {
	GetConn() net.Conn
	GetScanner() *bufio.Scanner
}

func send(com Communicator, original_data []byte) error {
	conn := com.GetConn()

	data := bytes.ReplaceAll(original_data, []byte("\r\n"), []byte("\\r\\n"))
	data = bytes.ReplaceAll(data, []byte("\n"), []byte("\\n"))
	if _, err := conn.Write(data); err != nil {
		return err
	}
	if _, err := conn.Write([]byte("\n")); err != nil {
		return err
	}

	return nil
}

func receive(com Communicator) ([]byte, error) {
	scanner := com.GetScanner()
	if !scanner.Scan() {
		return nil, scanner.Err()
	}
	data := scanner.Bytes()

	data = bytes.ReplaceAll(data, []byte("\\r\\n"), []byte("\r\n"))
	data = bytes.ReplaceAll(data, []byte("\\n"), []byte("\n"))

	return data, nil
}

// 予備校に該当するよい英単語がないためローマ字表記
type YobikouServer struct {
	conn    net.Conn
	scanner *bufio.Scanner
}

func NewYobikouServer() (YobikouServer, error) {
	tcpAddr, err := net.ResolveTCPAddr(PROTOCOL, PORT)
	if err != nil {
		return YobikouServer{}, err
	}

	listener, err := net.ListenTCP(PROTOCOL, tcpAddr)
	if err != nil {
		return YobikouServer{}, err
	}

	fmt.Println("中学側に接続します……")

	conn, err := listener.Accept()
	if err != nil {
		return YobikouServer{}, err
	}

	scanner := bufio.NewScanner(conn)

	fmt.Printf("接続しました．: %v\n", conn.RemoteAddr())

	return YobikouServer{conn, scanner}, nil
}

func (yobikou YobikouServer) GetConn() net.Conn {
	return yobikou.conn
}

func (yobikou YobikouServer) GetScanner() *bufio.Scanner {
	return yobikou.scanner
}

func (yobikou YobikouServer) Close() error {
	return yobikou.conn.Close()
}

func (yobikou YobikouServer) Send(data []byte) error {
	return send(yobikou, data)
}

func (yobikou YobikouServer) Receive() ([]byte, error) {
	return receive(yobikou)
}

// 予備校に合わせ中学校もローマ字で
type ChugakuClient struct {
	conn    net.Conn
	scanner *bufio.Scanner
}

func NewChugakuClient(serverAddr string) (ChugakuClient, error) {
	fmt.Println("予備校に接続します……")

	conn, err := net.DialTimeout(PROTOCOL, serverAddr+PORT, DEADLINE)
	if err != nil {
		return ChugakuClient{}, err
	}

	scanner := bufio.NewScanner(conn)

	fmt.Printf("接続しました．: %v\n", conn.RemoteAddr())

	return ChugakuClient{conn, scanner}, nil
}

func (chugaku ChugakuClient) GetConn() net.Conn {
	return chugaku.conn
}

func (chugaku ChugakuClient) GetScanner() *bufio.Scanner {
	return chugaku.scanner
}

func (chugaku ChugakuClient) Close() error {
	return chugaku.conn.Close()
}

func (chugaku ChugakuClient) Send(data []byte) error {
	return send(chugaku, data)
}

func (chugaku ChugakuClient) Receive() ([]byte, error) {
	return receive(chugaku)
}
