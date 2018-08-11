package main

import (
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

type Entityable interface {
}

type Entity struct {
	id     ID
	pos    complex128
	name   string
	vel    complex128
	size   float64
	weight float64
}

type Entities []Entity

type Character struct {
	Entity
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
}

func (v *Character) attackBy(a Character) {
	v.currHealth -= a.damage
}

func (ents *Entities) upd() {
	return
}

func (p *Player) update(theMap *Mapt, entities *[]NPC, moveDirection complex128) {
	if cmplx.Abs(moveDirection) > 1 {
		moveDirection = moveDirection / complex(cmplx.Abs(moveDirection), 0)
	}
	p.vel = approach(p.vel, moveDirection/5)
	p.pos += p.vel
	for i := range *entities {
		(*entities)[i].pos += (*entities)[i].vel
	}
	for i, e := range *entities {
		colDir := p.pos - e.pos
		colDep := (p.size+e.size)/2 - cmplx.Abs(colDir)
		if colDep > 0 {
			(*entities)[i].currHealth -= p.damage
			p.pos += cmul(colDir, colDep)
			(*entities)[i].pos -= cmul(colDir, colDep)
		}
	}
	for i, e := range *entities {
		if e.aggro {
			diff := p.pos - e.pos
			(*entities)[i].vel = approach(e.vel, diff/complex(cmplx.Abs(diff), 0)/10)
		}
		for j, o := range *entities {
			colDir := e.pos - o.pos
			colDep := (e.size+o.size)/2 - cmplx.Abs(colDir)
			if i != j && colDep > 0 {
				(*entities)[i].pos += cmul(colDir, colDep)
				(*entities)[j].pos -= cmul(colDir, colDep)
				(*entities)[i].vel = (e.vel + o.vel) / 2
				(*entities)[j].vel = (e.vel + o.vel) / 2
			}
		}
	}
}

//func (p *Player) update(theMap *Mapt, actors *[]NPC, moveDirection complex128) {
//	if cmplx.Abs(moveDirection) > 1 {
//		moveDirection = moveDirection / complex(cmplx.Abs(moveDirection), 0)
//	}
//	p.vel = approach(p.vel, moveDirection/5)
//	newPos := p.pos + p.vel
//
//	px, py := parts(p.pos)
//
//	pxf, pxc, pyf, pyc := int(px-1), int(px+1), int(py-1), int(py+1)
//
//	for _, a := range *actors {
//		colDir := newPos-a.pos
//		if cmplx.Abs(colDir) < (p.size+a.size)/2 {
//
//		}
//	}
//	for y := pyf; y <= pyc; y++ {
//		for x := pxf; x <= pxc; x++ {
//			if IS_SOLID[theMap[y][x].tileID] {
//
//				if cmplx.Abs(newPos-complex(float64(x), float64(y))) < (p.size+1)/2 {
//					//p.vel = 0+0i
//					//newPos = p.pos
//				}
//				//drawTile(textures[16], float64(x)-px+maxTiles/2, float64(y)-py+maxTiles/2)
//			}
//		}
//	}
//	p.pos = newPos
//}

func testEnemyNPC(pos complex128, id int) NPC {
	return NPC{Character{Entity: Entity{id: makeActorID(id), pos: pos, name: "TestEnemy", size: 0.8}, maxHealth: 10, currHealth: 10}, true}
}

func dummyNPC(x, y int) NPC {
	return NPC{Character{Entity: Entity{id: DUMMY, pos: complex(float64(x), float64(y)), name: "dummy"}, maxHealth: 10, currHealth: 10}, true}
}

func approach(vel, target complex128) complex128 {
	return (vel*4 + target) / 5
}
