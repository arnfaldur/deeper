package main

import (
	"fmt"
	"math/cmplx"
)

//Direction
const (
	UP = iota
	DOWN
	LEFT
	RIGHT
)

//Damage Types
const (
	dtKinetic = iota
	dtMagic
)

//Attributes
const (
	STR = iota
	DEX
	CON
	INT
	WIS
	CHA
	SPD
)

type Entity struct {
	pos    complex128
	name   string
	vel    complex128
	size   float64
	weight float64
}

type Character struct {
	Entity
	id                    ID
	maxHealth, currHealth int
	attributes            [7]int
	damage                int
}

type Player struct {
	Character
}

type NPC struct {
	Character
	aggro bool
}

func (n *NPC) update(p *Player) {
	if n.aggro {
		diff := p.pos - n.pos
		n.pos += diff / complex(cmplx.Abs(diff), 0) * 0.03
	}
}

func (n NPC) isAtPos(xpos, ypos int) bool {
	return (int(real(n.pos)) == xpos) && (int(imag(n.pos)) == ypos)
}

func (p *Player) termupdate(floor *Mapt, others *[]NPC, action int) {
	xpos := int(real(p.pos))
	ypos := int(imag(p.pos))

	switch action {
	case UP:
		ypos -= 1
	case LEFT:
		xpos -= 1
	case DOWN:
		ypos += 1
	case RIGHT:
		xpos += 1
	}

	if DEBUGLOGGING {
		fmt.Print("Action: ")
		switch action {
		case UP:
			fmt.Println("UP")
		case LEFT:
			fmt.Println("LEFT")
		case DOWN:
			fmt.Println("DOWN")
		case RIGHT:
			fmt.Println("RIGHT")
		}
		fmt.Println("xpos", xpos)
		fmt.Println("ypos", ypos)
	}

	p.move(xpos, ypos, floor, others)
}

func (a *Character) attack(v *Character) {
	v.currHealth -= a.damage
}

func (p *Character) move(xpos, ypos int, floor *Mapt, others *[]NPC) {

	var t = (*floor)[ypos][xpos]
	if DEBUGLOGGING {
		fmt.Println("IS SOLID", IS_SOLID[t.tileID])
	}
	if !IS_SOLID[t.tileID] {
		for _, o := range *others {
			if o.isAtPos(xpos, ypos) {
				//p.attack(&(*others)[i])
				break
			}
		}
		p.pos = complex(float64(xpos), float64(ypos))
		if DEBUGLOGGING {
			fmt.Println("xpos,ypos: ", xpos, ypos)
		}
	}
}

func (p *Player) update(theMap *Mapt, actors *[]NPC, moveDirection complex128) {
	if cmplx.Abs(moveDirection) > 1 {
		moveDirection = moveDirection / complex(cmplx.Abs(moveDirection), 0)
	}
	p.vel = approach(p.vel, moveDirection/5)
	newPos := p.pos + p.vel

	px, py := parts(p.pos)

	pxf, pxc := int(px-1), int(px+1)
	pyf, pyc := int(py-1), int(py+1)

	for _, a := range *actors {
		if cmplx.Abs(newPos-a.pos) < (p.size+a.size)/2 {

		}
	}

	for y := pyf; y <= pyc; y++ {
		for x := pxf; x <= pxc; x++ {
			if IS_SOLID[theMap[y][x].tileID] {

				if cmplx.Abs(newPos-complex(float64(x), float64(y))) < (p.size+1)/2 {
					//p.vel = 0+0i
					//newPos = p.pos
				}
				//drawTile(textures[16], float64(x)-px+MAX_TILES/2, float64(y)-py+MAX_TILES/2)
			}
		}
	}

	p.pos = newPos
}

func testEnemyNPC(pos complex128, id int) NPC {
	return NPC{Character{Entity: Entity{pos: pos, name: "TestEnemy", size: 0.8}, id: makeActorID(id), maxHealth: 10, currHealth: 10}, true}
}

func dummyNPC(x, y int) NPC {
	return NPC{Character{Entity: Entity{pos: complex(float64(x), float64(y)), name: "dummy"}, id: DUMMY, maxHealth: 10, currHealth: 10}, true}
}

func approach(vel, target complex128) complex128 {
	return (vel*4 + target) / 5
}
