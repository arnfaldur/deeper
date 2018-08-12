package main

import (
	"math"
	"math/cmplx"
	"math/rand"
)

type Entity struct {
	id        ID
	pos       complex128
	name      string
	collision bool
	solid     bool
	vel       complex128
	size      float64
	weight    float64
}

type Tile struct {
	Entity
	npcsOnTile []*NPC
}

func NewTile(id ID, x float64, y float64) Tile {
	return Tile{Entity: Entity{id: id, pos: complex(x, y), collision: isSolid[id], solid: isSolid[id]}}
}

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

func (v *Character) attackBy(a Character) {
	v.currHealth -= a.damage
}

func getNPCsNear(pos complex128, radius float64) []*NPC {
	var npcs []*NPC
	for _, t := range vicinity(pos, radius) {
		if t[0] >= 0 && t[0] < MAPSIZE && t[1] >= 0 && t[1] < MAPSIZE {
			for i := range theMap[t[0]][t[1]].npcsOnTile {
				npcs = append(npcs, theMap[t[0]][t[1]].npcsOnTile[i])
			}
		}
	}
	return npcs
}

func (e *Entity) entityCollision() {

}

func (e *Entity) tileCollide(theMap *Mapt) {
	px, py := parts(e.pos + e.vel)

	pxf, pxr, pxc, pyf, pyr, pyc := int(px), int(px+0.5), int(math.Nextafter(px+1, math.Inf(-1))), int(py), int(py+0.5), int(math.Nextafter(py+1, math.Inf(-1)))
	any := false
	toWall := e.size / 2
	if isSolid[theMap[pyf][pxr].id] && toWall >= math.Abs(py-0.5-float64(pyf)) {
		e.vel = complex(real(e.vel), math.Max(0, imag(e.vel)))
		//e.pos = complex(real(e.pos), float64(pyf)+0.5+toWall)
		any = true
	} else if isSolid[theMap[pyc][pxr].id] && toWall >= math.Abs(py+0.5-float64(pyc)) {
		e.vel = complex(real(e.vel), math.Min(0, imag(e.vel)))
		//e.pos = complex(real(e.pos), float64(pyc)-0.5-toWall)
		any = true
	}
	if isSolid[theMap[pyr][pxf].id] && toWall >= math.Abs(px-0.5-float64(pxf)) {
		e.vel = complex(math.Max(0, real(e.vel)), imag(e.vel))
		//e.pos = complex(float64(pxf)+0.5+toWall, imag(e.pos))
		any = true
	} else if isSolid[theMap[pyr][pxc].id] && toWall >= math.Abs(px+0.5-float64(pxc)) {
		e.vel = complex(math.Min(0, real(e.vel)), imag(e.vel))
		//e.pos = complex(float64(pxc)-0.5-toWall, imag(e.pos))
		any = true
	}
	if !any {
		fs := [4]float64{py - 0.5, py + 0.5, px - 0.5, px + 0.5}
		is := [4]int{pyf, pyc, pxf, pxc}
		for y := 0; y < 2; y++ {
			for x := 2; x < 4; x++ {
				colDir := complex(fs[x]-float64(is[x]), fs[y]-float64(is[y]))
				if isSolid[theMap[is[y]][is[x]].id] && toWall > cmplx.Abs(colDir) {
					colDep := toWall - cmplx.Abs(colDir)
					e.vel += cmplx.Rect(colDep/2, cmplx.Phase(colDir))
					//e.pos += cmplx.Rect(colDep, cmplx.Phase(colDir))
					break
				}
			}
		}
	}
}

func (c *Character) npcCollide(npcs *[]NPC) {
	netForce := 0 + 0i
	neighbours := getNPCsNear(c.pos, (c.size+MAXCHARSIZE+sqrt2)/2)
	for i, n := range neighbours {
		colDir := (c.pos + c.vel) - (n.pos)
		colDep := (c.size+n.size)/2 - cmplx.Abs(colDir)
		if *c != n.Character && colDep > 0 {
			//nudge := cmul(colDir, colDep)
			nudge := cmplx.Rect(colDep/2, cmplx.Phase(colDir))
			//println(colDep)
			//c.pos += nudge
			//(*npcs)[i].pos -= nudge
			if c.id == PLAYER {
				neighbours[i].currHealth -= c.damage
			} else {
				netForce += nudge
				//c.vel = c.vel + nudge
				//(*npcs)[i].vel = n.vel - nudge
			}
		}
	}
	c.vel += netForce
}

func (e *Entity) findCollisions(theMap Mapt) {
	for _, t := range vicinity(e.pos, e.size+MAXCHARSIZE) {
		for _, n := range theMap[t[0]][t[1]].npcsOnTile {
			if cmplx.Abs(e.pos+e.vel-n.pos+n.vel) <= (e.size+n.size)/2 {

			}
		}
	}
}

func findAllCollisions(theMap Mapt, p Player, npcs []NPC) {
	p.findCollisions(theMap)
	for _, n := range npcs {
		n.findCollisions(theMap)
	}
}

func dealWithCollisions(theMap *Mapt, p *Player, npcs *[]NPC, moveDirection complex128) {
	if cmplx.Abs(moveDirection) > 1 {
		moveDirection = cmplxNorm(moveDirection)
	}
	p.vel = approach(p.vel, moveDirection/5) * complex(timeDilation, 0)
	for i, e := range *npcs {
		if e.aggro {
			diff := p.pos - e.pos
			(*npcs)[i].vel = cmul(approach(e.vel, cmplxNorm(diff)/10+cmplx.Rect(0.01, (rand.Float64()*math.Pi*2)-math.Pi)), timeDilation)
		}
	}
	p.npcCollide(npcs)
	for i := range *npcs {
		(*npcs)[i].npcCollide(npcs)
	}
	p.tileCollide(theMap)
	for i := range *npcs {
		(*npcs)[i].tileCollide(theMap)
	}
	p.pos += p.vel
	for i := range *npcs {
		(*npcs)[i].pos += (*npcs)[i].vel
	}
}

func testEnemyNPC(pos complex128, id int) NPC {
	return NPC{Character{Entity: Entity{id: makeActorID(id), pos: pos, name: "TestEnemy", size: 0.8}, maxHealth: 10, currHealth: 10}, true}
}

func dummyNPC(x, y int) NPC {
	return NPC{Character{Entity: Entity{id: DUMMY, pos: complex(float64(x), float64(y)), name: "dummy"}, maxHealth: 10, currHealth: 10}, true}
}

func approach(vel, target complex128) complex128 {
	return (vel*4 + target) / 5
}
